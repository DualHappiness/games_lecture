class PrtMaterial extends Material {

    constructor(color, vertexShader, fragmentShader) {

        super({
            'uSampler': { type: 'texture', value: color },
            'uPrecomputeL': { type: 'updatedInRealTime', value: null },
        }, ['aPrecomputeLT'], vertexShader, fragmentShader, null);
    }
}

async function buildPrtMaterial(color, vertexPath, fragmentPath) {


    let vertexShader = await getShaderString(vertexPath);
    let fragmentShader = await getShaderString(fragmentPath);

    return new PrtMaterial(color, vertexShader, fragmentShader);
}