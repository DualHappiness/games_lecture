class SSRMaterial extends Material {
    constructor(diffuseMap, specularMap, light, camera, vertexShader, fragmentShader) {
        let lightIntensity = light.mat.GetIntensity();
        let lightVP = light.CalcLightVP();
        let lightDir = light.CalcShadingDirection();

        super({
            'uLightRadiance': { type: '3fv', value: lightIntensity },
            'uLightDir': { type: '3fv', value: lightDir },

            'uGDiffuse': { type: 'texture', value: camera.fbo.textures[0] },
            'uGNormalWorld': { type: 'texture', value: camera.fbo.textures[2] },
            'uGShadow': { type: 'texture', value: camera.fbo.textures[3] },
            'uGPosWorld': { type: 'texture', value: camera.fbo.textures[4] },

            'uGDepth[0]': { type: 'texture', value: camera.fbo.textures[1] },
            'uGDepth[1]': { type: 'texture', value: camera.mipmapFbos[0].textures[0] },
            'uGDepth[2]': { type: 'texture', value: camera.mipmapFbos[1].textures[0] },
            'uGDepth[3]': { type: 'texture', value: camera.mipmapFbos[2].textures[0] },
        }, [], vertexShader, fragmentShader);
    }
}

async function buildSSRMaterial(diffuseMap, specularMap, light, camera, vertexPath, fragmentPath) {
    let vertexShader = await getShaderString(vertexPath);
    let fragmentShader = await getShaderString(fragmentPath);

    return new SSRMaterial(diffuseMap, specularMap, light, camera, vertexShader, fragmentShader);
}