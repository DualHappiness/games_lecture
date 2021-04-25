class PrtMaterial extends Material {
    constructor(vertexShader, fragmentShader) {

        super({
            'uPrecomputeL[0]': { type: 'updatedInRealTime', value: null },
            'uPrecomputeL[1]': { type: 'updatedInRealTime', value: null },
            'uPrecomputeL[2]': { type: 'updatedInRealTime', value: null },
        }, ['aPrecomputeLT'], vertexShader, fragmentShader, null);
    }
}

async function buildPrtMaterial(vertexPath, fragmentPath) {


    let vertexShader = await getShaderString(vertexPath);
    let fragmentShader = await getShaderString(fragmentPath);

    return new PrtMaterial(vertexShader, fragmentShader);
}