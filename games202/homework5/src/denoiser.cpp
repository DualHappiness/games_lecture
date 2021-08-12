#include "denoiser.h"

Denoiser::Denoiser() : m_useTemportal(false) {}

void Denoiser::Reprojection(const FrameInfo &frameInfo) {
    int height = m_accColor.m_height;
    int width = m_accColor.m_width;
    Matrix4x4 preWorldToScreen =
        m_preFrameInfo.m_matrix[m_preFrameInfo.m_matrix.size() - 1];
    Matrix4x4 preWorldToCamera =
        m_preFrameInfo.m_matrix[m_preFrameInfo.m_matrix.size() - 2];
#pragma omp parallel for
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            // TODO: Reproject
            m_valid(x, y) = false;
            m_misc(x, y) = Float3(0.f);
        }
    }
    std::swap(m_misc, m_accColor);
}

void Denoiser::TemporalAccumulation(const Buffer2D<Float3> &curFilteredColor) {
    int height = m_accColor.m_height;
    int width = m_accColor.m_width;
    int kernelRadius = 3;
#pragma omp parallel for
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            // TODO: Temporal clamp
            Float3 color = m_accColor(x, y);
            // TODO: Exponential moving average
            float alpha = 1.0f;
            m_misc(x, y) = Lerp(color, curFilteredColor(x, y), alpha);
        }
    }
    std::swap(m_misc, m_accColor);
}

Buffer2D<Float3> Denoiser::Filter(const FrameInfo &frameInfo) {
    int height = frameInfo.m_beauty.m_height;
    int width = frameInfo.m_beauty.m_width;
    Buffer2D<Float3> filteredImage = CreateBuffer2D<Float3>(width, height);
    int kernelRadius = 16;
#pragma omp parallel for
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            // Joint bilateral filter
            float sum_of_weights = 0.0f;
            Float3 color = 0.0f;
            for (int i = -kernelRadius; i < kernelRadius; i++) {
                for (int j = -kernelRadius; j < kernelRadius; j++) {
                    int x2 = x + i;
                    int y2 = y + j;
                    if (x2 < 0 || x2 > width || y2 < 0 || y2 > height) {
                        continue;
                    }
                    float temp = -(Sqr(i) + Sqr(j)) / (2.0 * Sqr(m_sigmaCoord));
                    temp += -SqrLength(frameInfo.m_beauty(x, y) -
                                       frameInfo.m_beauty(x2, y2)) /
                            (2.0 * Sqr(m_sigmaColor));
                    temp += -SafeAcos(Dot(frameInfo.m_normal(x, y),
                                          frameInfo.m_normal(x2, y2))) /
                            (2.0 * Sqr(m_sigmaNormal));
                    temp += -Dot(frameInfo.m_normal(x, y),
                                 (frameInfo.m_beauty(x, y) - frameInfo.m_beauty(x2, y2)) /
                                     Length(frameInfo.m_beauty(x, y) -
                                            frameInfo.m_beauty(x2, y2))) /
                            (2.0 * Sqr(m_sigmaPlane));
                    temp = exp(temp);
                    sum_of_weights += temp;
                    color += frameInfo.m_beauty(x2, y2) * temp;
                }
            }
            filteredImage(x, y) = color / sum_of_weights;
        }
    }
    return filteredImage;
}

void Denoiser::Init(const FrameInfo &frameInfo, const Buffer2D<Float3> &filteredColor) {
    m_accColor.Copy(filteredColor);
    int height = m_accColor.m_height;
    int width = m_accColor.m_width;
    m_misc = CreateBuffer2D<Float3>(width, height);
    m_valid = CreateBuffer2D<bool>(width, height);
}

void Denoiser::Maintain(const FrameInfo &frameInfo) { m_preFrameInfo = frameInfo; }

Buffer2D<Float3> Denoiser::ProcessFrame(const FrameInfo &frameInfo) {
    // Filter current frame
    Buffer2D<Float3> filteredColor;
    filteredColor = Filter(frameInfo);

    // Reproject previous frame color to current
    if (m_useTemportal) {
        Reprojection(frameInfo);
        TemporalAccumulation(filteredColor);
    } else {
        Init(frameInfo, filteredColor);
    }

    // Maintain
    Maintain(frameInfo);
    if (!m_useTemportal) {
        m_useTemportal = true;
    }
    return m_accColor;
}
