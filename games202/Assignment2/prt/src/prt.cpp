#include <nori/integrator.h>
#include <nori/scene.h>
#include <nori/ray.h>
#include <filesystem/resolver.h>
#include <sh/spherical_harmonics.h>
#include <sh/default_image.h>
#include <Eigen/Core>
#include <fstream>
#include <random>
#include <stb_image.h>

NORI_NAMESPACE_BEGIN

namespace ProjEnv
{
    std::vector<std::unique_ptr<float[]>>
    LoadCubemapImages(const std::string &cubemapDir, int &width, int &height,
                      int &channel)
    {
        std::vector<std::string> cubemapNames{"negx.jpg", "posx.jpg", "posy.jpg",
                                              "negy.jpg", "posz.jpg", "negz.jpg"};
        std::vector<std::unique_ptr<float[]>> images(6);
        for (int i = 0; i < 6; i++)
        {
            std::string filename = cubemapDir + "/" + cubemapNames[i];
            int w, h, c;
            float *image = stbi_loadf(filename.c_str(), &w, &h, &c, 3);
            if (!image)
            {
                std::cout << "Failed to load image: " << filename << std::endl;
                exit(-1);
            }
            if (i == 0)
            {
                width = w;
                height = h;
                channel = c;
            }
            else if (w != width || h != height || c != channel)
            {
                std::cout << "Dismatch resolution for 6 images in cubemap" << std::endl;
                exit(-1);
            }
            images[i] = std::unique_ptr<float[]>(image);
            int index = (0 * 128 + 0) * channel;
            // std::cout << images[i][index + 0] << "\t" << images[i][index + 1] << "\t"
            //           << images[i][index + 2] << std::endl;
        }
        return images;
    }

    const Eigen::Vector3f cubemapFaceDirections[6][3] = {
        {{0, 0, 1}, {0, -1, 0}, {-1, 0, 0}},  // negx
        {{0, 0, 1}, {0, -1, 0}, {1, 0, 0}},   // posx
        {{1, 0, 0}, {0, 0, -1}, {0, -1, 0}},  // negy
        {{1, 0, 0}, {0, 0, 1}, {0, 1, 0}},    // posy
        {{-1, 0, 0}, {0, -1, 0}, {0, 0, -1}}, // negz
        {{1, 0, 0}, {0, -1, 0}, {0, 0, 1}},   // posz
    };

    float CalcPreArea(const float &x, const float &y)
    {
        return std::atan2(x * y, std::sqrt(x * x + y * y + 1.0));
    }

    float CalcArea(const float &u_, const float &v_, const int &width,
                   const int &height)
    {
        // transform from [0..res - 1] to [- (1 - 1 / res) .. (1 - 1 / res)]
        // ( 0.5 is for texel center addressing)
        float u = (2.0 * (u_ + 0.5) / width) - 1.0;
        float v = (2.0 * (v_ + 0.5) / height) - 1.0;

        // shift from a demi texel, mean 1.0 / size  with u and v in [-1..1]
        float invResolutionW = 1.0 / width;
        float invResolutionH = 1.0 / height;

        // u and v are the -1..1 texture coordinate on the current face.
        // get projected area for this texel
        float x0 = u - invResolutionW;
        float y0 = v - invResolutionH;
        float x1 = u + invResolutionW;
        float y1 = v + invResolutionH;
        float angle = CalcPreArea(x0, y0) - CalcPreArea(x0, y1) -
                      CalcPreArea(x1, y0) + CalcPreArea(x1, y1);

        return angle;
    }

    // template <typename T> T ProjectSH() {}

    template <size_t SHOrder>
    std::vector<Eigen::Array3f> PrecomputeCubemapSH(const std::vector<std::unique_ptr<float[]>> &images,
                                                    const int &width, const int &height,
                                                    const int &channel)
    {
        std::vector<Eigen::Vector3f> cubemapDirs;
        cubemapDirs.reserve(6 * width * height);
        for (int i = 0; i < 6; i++)
        {
            Eigen::Vector3f faceDirX = cubemapFaceDirections[i][0];
            Eigen::Vector3f faceDirY = cubemapFaceDirections[i][1];
            Eigen::Vector3f faceDirZ = cubemapFaceDirections[i][2];
            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    float u = 2 * ((x + 0.5) / width) - 1;
                    float v = 2 * ((y + 0.5) / height) - 1;
                    Eigen::Vector3f dir = (faceDirX * u + faceDirY * v + faceDirZ).normalized();
                    cubemapDirs.push_back(dir);
                }
            }
        }
        constexpr int SHNum = (SHOrder + 1) * (SHOrder + 1);
        std::vector<Eigen::Array3f> SHCoeffiecents(SHNum);
        for (int i = 0; i < SHNum; i++)
            SHCoeffiecents[i] = Eigen::Array3f(0);
        for (int i = 1; i < 6; i++)
        {
            for (int y = 0; y < height; y++)
            {
                for (int x = 0; x < width; x++)
                {
                    Eigen::Vector3f dir = cubemapDirs[i * width * height + y * width + x];
                    int index = (y * width + x) * channel;
                    Eigen::Array3f Le(images[i][index + 0], images[i][index + 1],
                                      images[i][index + 2]);
                    double delta = CalcArea(static_cast<float>(x), static_cast<float>(y), width, height);
                    for (int l = 0; l <= SHOrder; l++)
                    {
                        for (int m = -l; m <= l; m++)
                        {
                            double sh = sh::EvalSH(l, m, dir.cast<double>().normalized());
                            SHCoeffiecents[sh::GetIndex(l, m)] += sh * delta * Le.array();
                        }
                    }
                }
            }
        }
        return SHCoeffiecents;
    }
}

class PRTIntegrator : public Integrator
{
public:
    static constexpr int SHOrder = 2;
    static constexpr int SHCoeffLength = (SHOrder + 1) * (SHOrder + 1);

    enum class Type
    {
        Unshadowed = 0,
        Shadowed = 1,
        Interreflection = 2
    };

    PRTIntegrator(const PropertyList &props)
    {
        /* No parameters this time */
        m_SampleCount = props.getInteger("PRTSampleCount", 100);
        m_CubemapPath = props.getString("cubemap");
        auto type = props.getString("type", "unshadowed");
        if (type == "unshadowed")
        {
            m_Type = Type::Unshadowed;
        }
        else if (type == "shadowed")
        {
            m_Type = Type::Shadowed;
        }
        else if (type == "interreflection")
        {
            m_Type = Type::Interreflection;
            m_Bounce = props.getInteger("bounce", 1);
        }
        else
        {
            throw NoriException("Unsupported type: %s.", type);
        }
    }

    virtual void preprocess(const Scene *scene) override
    {

        // Here only compute one mesh
        const auto mesh = scene->getMeshes()[0];
        // Projection environment
        auto cubePath = getFileResolver()->resolve(m_CubemapPath);
        auto lightPath = cubePath / "light.txt";
        auto transPath = cubePath / "transport.txt";
        std::ofstream lightFout(lightPath.str());
        std::ofstream fout(transPath.str());
        int width, height, channel;
        std::vector<std::unique_ptr<float[]>> images =
            ProjEnv::LoadCubemapImages(cubePath.str(), width, height, channel);
        std::vector<Eigen::Array3f> envCoeffs = ProjEnv::PrecomputeCubemapSH<SHOrder>(images, width, height, channel);
        m_LightCoeffs.resize(3, SHCoeffLength);
        for (int i = 0; i < envCoeffs.size(); i++)
        {
            lightFout << (envCoeffs)[i].x() << " " << (envCoeffs)[i].y() << " " << (envCoeffs)[i].z() << std::endl;
            m_LightCoeffs.col(i) = (envCoeffs)[i];
        }
        std::cout << "Computed light sh coeffs from: " << cubePath.str() << " to: " << lightPath.str() << std::endl;
        // Projection transport
        m_TransportSHCoeffs.resize(SHCoeffLength, mesh->getVertexCount());
        fout << mesh->getVertexCount() << std::endl;
        for (int i = 0; i < mesh->getVertexCount(); i++)
        {
            const Point3f &v = mesh->getVertexPositions().col(i);
            const Normal3f &n = mesh->getVertexNormals().col(i);
            auto shFunc = [&](double phi, double theta) -> double {
                Eigen::Array3d d = sh::ToVector(phi, theta);
                const auto wi = Vector3f(d.x(), d.y(), d.z());
                auto H = std::max(0.0f, n.dot(wi));
                if (m_Type != Type::Unshadowed)
                {
                    const float BIAS = 0.0000001f;
                    Ray3f ray = Ray3f(v + wi * BIAS, wi);
                    auto V = scene->rayIntersect(ray) ? 0.0f : 1.0f;
                    return H * V;
                }
                return H;
            };
            auto shCoeff = sh::ProjectFunction(SHOrder, shFunc, m_SampleCount);
            for (int j = 0; j < shCoeff->size(); j++)
            {
                m_TransportSHCoeffs.col(i).coeffRef(j) = (*shCoeff)[j];
            }
        }
        if (m_Type == Type::Interreflection)
        {
            Eigen::MatrixXf temp;
            temp.resize(SHCoeffLength, mesh->getVertexCount());
            for (int i = 0; i < mesh->getVertexCount(); i++)
            {
                const Point3f &v = mesh->getVertexPositions().col(i);
                const Normal3f &n = mesh->getVertexNormals().col(i);
                auto shCoeff = InterRecuseHelp(scene, mesh, v, n, 0, m_Bounce);
                for (int j = 0; j < shCoeff->size(); j++)
                {
                    temp.col(i).coeffRef(j) = (*shCoeff)[j];
                }
            }
            for (int i = 0; i < mesh->getVertexCount(); i++)
            {
                for (int j = 0; j < SHCoeffLength; j++)
                {
                    m_TransportSHCoeffs.col(i).coeffRef(j) += temp.col(i).coeffRef(j);
                }
            }
        }

        // Save in face format
        for (int f = 0; f < mesh->getTriangleCount(); f++)
        {
            const MatrixXu &F = mesh->getIndices();
            uint32_t idx0 = F(0, f), idx1 = F(1, f), idx2 = F(2, f);
            for (int j = 0; j < SHCoeffLength; j++)
            {
                fout << m_TransportSHCoeffs.col(idx0).coeff(j) << " ";
            }
            fout << std::endl;
            for (int j = 0; j < SHCoeffLength; j++)
            {
                fout << m_TransportSHCoeffs.col(idx1).coeff(j) << " ";
            }
            fout << std::endl;
            for (int j = 0; j < SHCoeffLength; j++)
            {
                fout << m_TransportSHCoeffs.col(idx2).coeff(j) << " ";
            }
            fout << std::endl;
        }
        std::cout << "Computed SH coeffs"
                  << " to: " << transPath.str() << std::endl;
    }

    Color3f Li(const Scene *scene, Sampler *sampler, const Ray3f &ray) const
    {
        Intersection its;
        if (!scene->rayIntersect(ray, its))
            return Color3f(0.0f);

        const Eigen::Matrix<Vector3f::Scalar, SHCoeffLength, 1> sh0 = m_TransportSHCoeffs.col(its.tri_index.x()),
                                                                sh1 = m_TransportSHCoeffs.col(its.tri_index.y()),
                                                                sh2 = m_TransportSHCoeffs.col(its.tri_index.z());
        const Eigen::Matrix<Vector3f::Scalar, SHCoeffLength, 1> rL = m_LightCoeffs.row(0), gL = m_LightCoeffs.row(1), bL = m_LightCoeffs.row(2);

        Color3f c0 = Color3f(rL.dot(sh0), gL.dot(sh0), bL.dot(sh0)),
                c1 = Color3f(rL.dot(sh1), gL.dot(sh1), bL.dot(sh1)),
                c2 = Color3f(rL.dot(sh2), gL.dot(sh2), bL.dot(sh2));

        const Vector3f &bary = its.bary;
        Color3f c = bary.x() * c0 + bary.y() * c1 + bary.z() * c2;
        return c;
    }

    std::string toString() const
    {
        return tfm::format(
            "PRTIntegrator[\n"
            "   type=%s\n"
            "]",
            static_cast<std::underlying_type<Type>::type>(m_Type));
    }

private:
    Type m_Type;
    int m_Bounce = 1;
    int m_SampleCount = 100;
    std::string m_CubemapPath;
    Eigen::MatrixXf m_TransportSHCoeffs;
    Eigen::MatrixXf m_LightCoeffs;

    std::unique_ptr<std::vector<double>> InterRecuseHelp(const Scene *scene, const Mesh *mesh, const Point3f &p, const Normal3f &n, int curDepth, const int maxDepth)
    {
        if (curDepth >= maxDepth)
        {
            std::unique_ptr<std::vector<double>> sh(new std::vector<double>());
            sh->assign(sh::GetCoefficientCount(SHOrder), 0.0);
            return sh;
        }

        auto shFunc = [&](double phi, double theta) -> std::unique_ptr<std::vector<double>> {
            Eigen::Array3d d = sh::ToVector(phi, theta);
            const auto wi = Vector3f(d.x(), d.y(), d.z());

            std::unique_ptr<std::vector<double>> sh(new std::vector<double>());
            sh->assign(sh::GetCoefficientCount(SHOrder), 0.0);
            const float BIAS = 0.000000001f;
            Ray3f ray = Ray3f(p + wi * BIAS, wi);
            Intersection inter;
            if (scene->rayIntersect(ray, inter))
            {
                Point3f inter_p = inter.tri_index;
                Point3f bary = inter.bary;
                Normal3f inter_n = mesh->getVertexNormals().col(p.x()) * bary.x() + mesh->getVertexNormals().col(p.y()) * bary.y() + mesh->getVertexNormals().col(p.z()) * bary.z();
                inter_n.normalize();

                if (inter_n.dot(-wi) > 0.0)
                {
                    auto inderict = InterRecuseHelp(scene, mesh, inter_p, inter_n, curDepth + 1, maxDepth);

                    // distribute in hemisphere
                    auto H = std::max(0.0f, n.dot(wi)) / (2.0 * M_PI);

                    for (int i = 0; i < sh->size(); i++)
                    {
                        auto inter_shi = m_TransportSHCoeffs.col(p.x()).coeffRef(i) * bary.x() + m_TransportSHCoeffs.col(p.y()).coeffRef(i) * bary.y() + m_TransportSHCoeffs.col(p.z()).coeffRef(i) * bary.z();
                        (*sh)[i] += ((*inderict)[i] + inter_shi) * H;
                    }
                }
            }
            return sh;
        };
        return InterProjectFunction(SHOrder, shFunc, m_SampleCount);
    }

    std::unique_ptr<std::vector<double>> InterProjectFunction(
        int order, const std::function<std::unique_ptr<std::vector<double>>(double, double)> &func, int sample_count)
    {

        // This is the approach demonstrated in [1] and is useful for arbitrary
        // functions on the sphere that are represented analytically.
        const int sample_side = static_cast<int>(floor(sqrt(sample_count)));
        std::unique_ptr<std::vector<double>> coeffs(new std::vector<double>());
        coeffs->assign(sh::GetCoefficientCount(order), 0.0);

        // generate sample_side^2 uniformly and stratified samples over the sphere
        std::random_device rd;
        std::mt19937 gen(rd());
        std::uniform_real_distribution<> rng(0.0, 1.0);
        for (int t = 0; t < sample_side; t++)
        {
            for (int p = 0; p < sample_side; p++)
            {
                double alpha = (t + rng(gen)) / sample_side;
                double beta = (p + rng(gen)) / sample_side;
                // See http://www.bogotobogo.com/Algorithms/uniform_distribution_sphere.php
                double phi = 2.0 * M_PI * beta;
                double theta = acos(2.0 * alpha - 1.0);

                // evaluate the analytic function for the current spherical coords
                auto shCoeffs = func(phi, theta);

                for (int i = 0; i < shCoeffs->size(); i++)
                {
                    (*coeffs)[i] += (*shCoeffs)[i];
                }
            }
        }

        // scale by the probability of a particular sample, which is
        // 4pi/sample_side^2. 4pi for the surface area of a unit sphere, and
        // 1/sample_side^2 for the number of samples drawn uniformly.
        double weight = 4.0 * M_PI / (sample_side * sample_side);
        for (unsigned int i = 0; i < coeffs->size(); i++)
        {
            (*coeffs)[i] *= weight;
        }

        return coeffs;
    }
};

NORI_REGISTER_CLASS(PRTIntegrator, "prt");
NORI_NAMESPACE_END