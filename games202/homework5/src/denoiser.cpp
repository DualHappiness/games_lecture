#include "denoiser.h"

Denoiser::Denoiser() : m_useTemportal(false) {}

void Denoiser::Reprojection(const FrameInfo &frameInfo) {
    int height = m_accColor.m_height;
    int width = m_accColor.m_width;
    Matrix4x4 preWorldToScreen =
        m_preFrameInfo.m_matrix[m_preFrameInfo.m_matrix.size() - 1];
#pragma omp parallel for
    for (int y = 0; y < height; y++) {
        for (int x = 0; x < width; x++) {
            // Reproject
            auto id = frameInfo.m_id(x, y);
            m_valid(x, y) = false;
            m_misc(x, y) = Float3(0.f);
            if (id == -1) {
                continue;
            }
            auto curWorldPos = frameInfo.m_position(x, y);
            auto curWorldToLocal = Inverse(frameInfo.m_matrix[id]);
            auto preLocalToWorld = m_preFrameInfo.m_matrix[id];
            auto transform = preWorldToScreen * preLocalToWorld * curWorldToLocal;
            auto preScreenPos = transform(curWorldPos, Float3::Point) - 0.5f;

            if (preScreenPos.x >= 0 && preScreenPos.x < width && preScreenPos.y >= 0 &&
                preScreenPos.y < height) {
                auto preId = m_preFrameInfo.m_id(preScreenPos.x, preScreenPos.y);
                if (preId == id) {
                    m_valid(x, y) = true;
                    m_misc(x, y) = m_accColor(preScreenPos.x, preScreenPos.y);
                }
            }
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
            Float3 average = 0.0f;
            float n = 0.0f;
            for (int i = -kernelRadius; i < kernelRadius; i++) {
                for (int j = -kernelRadius; j < kernelRadius; j++) {
                    if (x + i < 0 || y + j < 0 || x + i >= width || y + j >= height) {
                        continue;
                    }
                    average += curFilteredColor(x + i, y + j);
                    n += 1.0f;
                }
            }
            average /= n;
            Float3 sigma = 0.0f;
            for (int i = -kernelRadius; i < kernelRadius; i++) {
                for (int j = -kernelRadius; j < kernelRadius; j++) {
                    if (x + i < 0 || y + j < 0 || x + i >= width || y + j >= height) {
                        continue;
                    }
                    auto diff = Abs(curFilteredColor(x + i, y + j) - average);
                    sigma += diff * diff;
                }
            }
            sigma /= n;

            // Temporal clamp
            Float3 color = Clamp(m_accColor(x, y), average - sigma * m_colorBoxK,
                                 average + sigma * m_colorBoxK);
            // Exponential moving average
            float alpha = m_valid(x, y) ? m_alpha : 1.0f;
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
                    if (x2 < 0 || x2 >= width || y2 < 0 || y2 >= height) {
                        continue;
                    }
                    float temp = -(Sqr(i) + Sqr(j)) / (2.0f * Sqr(m_sigmaCoord));
                    temp -=
                        SqrLength(frameInfo.m_beauty(x, y) - frameInfo.m_beauty(x2, y2)) /
                        (2.0f * Sqr(m_sigmaColor));

                    if (SqrLength(frameInfo.m_normal(x, y)) > 0 &&
                        SqrLength(frameInfo.m_normal(x2, y2)) > 0) {
                        temp -= Sqr(SafeAcos(Dot(frameInfo.m_normal(x, y),
                                                 frameInfo.m_normal(x2, y2)))) /
                                (2.0f * Sqr(m_sigmaNormal));
                    }

                    auto diff = frameInfo.m_position(x2, y2) - frameInfo.m_position(x, y);
                    auto len = Length(diff);
                    float position_ratio = 1.0f;
                    if (len > .0f) {
                        position_ratio = Sqr(Dot(frameInfo.m_normal(x, y), diff / len));
                    }
                    temp -= position_ratio * (2.0f * Sqr(m_sigmaPlane));

                    float weight = std::exp(temp);
                    // std::cout << weight << ' ' << temp << ' ' << std::endl;
                    sum_of_weights += weight;
                    color += frameInfo.m_beauty(x2, y2) * weight;
                }
            }
            filteredImage(x, y) = sum_of_weights == .0 ? .0f : color / sum_of_weights;
            // filteredImage(x, y) = frameInfo.m_beauty(x, y);
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
