class PhongMaterial extends Material {

    constructor(color, specular, lights, translate, scale, vertexShader, fragmentShader) {
        let uniforms = {
            // Phong
            'uSampler': { type: 'texture', value: color },
            'uKs': { type: '3fv', value: specular },
            
            'uLightNum' : { type: '1i', value: lights.length}
        };
        lights.forEach((light, i) => {
            let lightMVP = light.CalcLightMVP(translate, scale);
            let lightIntensity = light.mat.GetIntensity();
            uniforms[`uLightIntensity[${i}]`] = { type: '3fv', value: lightIntensity };
            // Shadow
            uniforms[`uShadowMap[${i}]`] = { type: 'texture', value: light.fbo };
            uniforms[`uLightMVP[${i}]`] = { type: 'matrix4fv', value: lightMVP };
        });


        super(uniforms, [], vertexShader, fragmentShader);
    }
}

async function buildPhongMaterial(color, specular, light, translate, scale, vertexPath, fragmentPath) {


    let vertexShader = await getShaderString(vertexPath);
    let fragmentShader = await getShaderString(fragmentPath);

    return new PhongMaterial(color, specular, light, translate, scale, vertexShader, fragmentShader);

}