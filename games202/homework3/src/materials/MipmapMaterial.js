class MipmapMaterial extends Material {
    constructor(camera, vertexShader, fragmentShader) {

        super({
            'uInvWidth': { type: '1f', value: 0.0 },
            'uInvHeight': { type: '1f', value: 0.0 },

            'uGDepth': { type: 'texture', value: null },
        }, [], vertexShader, fragmentShader, camera.fbo);
    }
}

async function buildMipmapMaterial(camera) {
    const vertexPath = './src/shaders/mipmapShader/mipmapVertex.glsl';
    let vertexShader = await getShaderString(vertexPath);

    const fragmentPath = '/src/shaders/mipmapShader/mipmapFragment.glsl';
    let fragmentShader = await getShaderString(fragmentPath);

    return new MipmapMaterial(camera, vertexShader, fragmentShader);
}