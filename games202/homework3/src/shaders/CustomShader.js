async function buildCustomShader(gl, vertexPath, fragmentPath, uniforms = [], attribs = []) {
    let vertexShader = await getShaderString(vertexPath);

    let fragmentShader = await getShaderString(fragmentPath);

    return new Shader(gl, vertexShader, fragmentShader, {
        uniforms,
        attribs,
    });
}