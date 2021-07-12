### 完成内容：
- 完成Emu MC方法计算
  ```cpp
  /// Emu_MC.cpp
  Vec3f IntegrateBRDF(Vec3f V, float roughness, float NdotV)
  {
      float A = 0.0;
      const int sample_count = 1024;
      Vec3f N = Vec3f(0.0, 0.0, 1.0);

      samplePoints sampleList = squareToCosineHemisphere(sample_count);
      for (int i = 0; i < sample_count; i++)
      {
          Vec3f L = sampleList.directions[i];
          Vec3f H = normalize(V + L);
          float pdf = sampleList.PDFs[i];

          float NdotL = dot(N, L);

          float D = DistributionGGX(N, H, roughness);
          float G = GeometrySmith(roughness, NdotV, NdotL);

          float fr = G * D / (4 * NdotL * NdotV);

          // To calculate (fr * ni) / p_o here
          float E = fr * NdotL / pdf;
          A += E;
      }

      return Vec3f(A / sample_count);
  }
  ```
- 完成Eavg 计算
  ```cpp
  /// Eavg_IS.cpp Eavg_MC.cpp
  Vec3f IntegrateEmu(Vec3f V, float roughness, float NdotV, Vec3f Ei)
  {
      return Ei * 2.0 * NdotV;
  }
  ```
- 实现pbr材质
    ```js
    float DistributionGGX(vec3 N, vec3 H, float roughness) {
        float a = roughness * roughness;
        float a2 = a * a;
        float NdotH = max(0.0, dot(H, N));
        float NdotH2 = NdotH * NdotH;

        float nom = a2;
        float denom = (NdotH2 * (a2 - 1.0) + 1.0);
        denom = PI * denom * denom;
        return nom / max(denom, 0.0001)
    }

    float GeometrySchlickGGX(float NdotV, float roughness) {
        // To calculate Smith G1 here
        float a = roughness;
        float k = (a * a) / 2.0;

        float nom = NdotV;
        float denom = NdotV * (1.0 - k) + k;

        return nom / denom;
    }

    float GeometrySmith(vec3 N, vec3 V, vec3 L, float roughness) {
        // To calculate Smith G here
        float ggx1 = GeometrySchlickGGX(dot(N, L), roughness);
        float ggx2 = GeometrySchlickGGX(dot(N, V), roughness);

        return ggx1 * ggx2;
    }

    vec3 fresnelSchlick(vec3 F0, vec3 V, vec3 H) {
        // To calculate Schlick F here
        return F0 + (1.0 - F0) * pow(1.0 - dot(H, V), 5.0);
    }
    ```
- 实现 Kulla-Conty 材质
    ```js
    vec3 MultiScatterBRDF(float NdotL, float NdotV) {
        vec3 albedo = pow(texture2D(uAlbedoMap, vTextureCoord).rgb, vec3(2.2));

        vec3 E_o = texture2D(uBRDFLut, vec2(NdotL, uRoughness)).xyz;
        vec3 E_i = texture2D(uBRDFLut, vec2(NdotV, uRoughness)).xyz;

        vec3 E_avg = texture2D(uEavgLut, vec2(0, uRoughness)).xyz;
        // copper
        vec3 edgetint = vec3(0.827, 0.792, 0.678);
        vec3 F_avg = AverageFresnel(albedo, edgetint);

        // To calculate fms and missing energy here
        vec3 f_ms = (1.0 - E_i) * (1.0 - E_o) / (PI * (1.0 - E_avg));
        vec3 f_add = F_avg * E_avg / (1.0 - F_avg * (1.0 - E_avg));

        return f_add * f_ms;
    }
    ```
- 重要性采样
  ```cpp
  /// Emu_IS.cpp
  if (NoL > 0)
  {
    // To calculate (fr * ni) / p_o here - Bonus 1
    float G = GeometrySmith(roughness, NoV, NoL);
    float weight = VoH * G / (NoV * NoH);
  }
  ```
- split sum
  ```cpp
  // Split Sum - Bonus 2
  float Fc = pow(1 - VoH, 5);
  A += (1 - Fc) * weight;
  B += Fc * weight;
  ```
  最终 split sum的图 就算是r+g使用结果也很奇怪 感觉应该是数值精度的问题 但未解决