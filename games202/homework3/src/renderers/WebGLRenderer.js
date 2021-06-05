const mipmapLevel = 3;
class WebGLRenderer {
    meshes = [];
    shadowMeshes = [];
    bufferMeshes = [];
    lights = [];
    mipmapShader = null;
    #points = null;
    #indices = null;

    constructor(gl, camera) {
        this.gl = gl;
        this.camera = camera;

        camera.mipmapFbos = [];
        for (let level = 0; level < mipmapLevel; level++) {
            const width = window.screen.width >> (level + 1);
            const height = window.screen.height >> (level + 1);
            const fbo = new FBO(gl, width, height);
            camera.mipmapFbos.push(fbo);

        }
        this.#points = gl.createBuffer();
        gl.bindBuffer(gl.ARRAY_BUFFER, this.#points);
        const positions = [
            -1.0, -1.0,
            1.0, -1.0,
            1.0, 1.0,
            -1.0, 1.0,
        ];
        gl.bufferData(gl.ARRAY_BUFFER, new Float32Array(positions), gl.STATIC_DRAW);
        gl.bindBuffer(gl.ARRAY_BUFFER, null);
        this.#indices = gl.createBuffer();
        const indices = [
            0, 1, 2,
            0, 2, 3,
        ];
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.#indices);
        gl.bufferData(gl.ELEMENT_ARRAY_BUFFER, new Uint16Array(indices), gl.STATIC_DRAW);
        gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, null);
        buildCustomShader(
            gl,
            './src/shaders/mipmapShader/mipmapVertex.glsl',
            './src/shaders/mipmapShader/mipmapFragment.glsl',
            ['uInvWidth', 'uInvHeight', 'uGDepth'],
            ['aCoord']
        ).then(shader => this.mipmapShader = shader);
    }

    addLight(light) {
        this.lights.push({
            entity: light,
            meshRender: new MeshRender(this.gl, light.mesh, light.mat)
        });
    }
    addMeshRender(mesh) { this.meshes.push(mesh); }
    addShadowMeshRender(mesh) { this.shadowMeshes.push(mesh); }
    addBufferMeshRender(mesh) { this.bufferMeshes.push(mesh); }

    render() {
        if (this.mipmapShader == null) { return; }
        console.assert(this.lights.length != 0, "No light");
        console.assert(this.lights.length == 1, "Multiple lights");
        var light = this.lights[0];

        const gl = this.gl;
        gl.clearColor(0.0, 0.0, 0.0, 1.0);
        gl.clearDepth(1.0);
        gl.enable(gl.DEPTH_TEST);
        gl.depthFunc(gl.LEQUAL);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);

        // Update parameters
        let lightVP = light.entity.CalcLightVP();
        let lightDir = light.entity.CalcShadingDirection();
        let updatedParamters = {
            "uLightVP": lightVP,
            "uLightDir": lightDir,
        };

        // Draw light
        light.meshRender.mesh.transform.translate = light.entity.lightPos;
        // light.meshRender.draw(this.camera, null, updatedParamters);

        // Shadow pass
        gl.bindFramebuffer(gl.FRAMEBUFFER, light.entity.fbo);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        for (let i = 0; i < this.shadowMeshes.length; i++) {
            this.shadowMeshes[i].draw(this.camera, light.entity.fbo, updatedParamters);
            // this.shadowMeshes[i].draw(this.camera);
        }
        // // return;

        // // Buffer pass
        gl.bindFramebuffer(gl.FRAMEBUFFER, this.camera.fbo);
        gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
        for (let i = 0; i < this.bufferMeshes.length; i++) {
            this.bufferMeshes[i].draw(this.camera, this.camera.fbo, updatedParamters);
            // this.bufferMeshes[i].draw(this.camera);
        }
        // build mipmap
        const textures = [
            this.camera.fbo.textures[1],
            this.camera.mipmapFbos[0].textures[0],
            this.camera.mipmapFbos[1].textures[0],
            this.camera.mipmapFbos[2].textures[0],
        ];
        // for (let level = 0; level < mipmapLevel; level++) {
        const program = this.mipmapShader.program;
        for (let level = 0; level < mipmapLevel; level++) {
            const width = window.screen.width >> (level + 1);
            const height = window.screen.height >> (level + 1);
            const fbo = this.camera.mipmapFbos[level];

            gl.bindFramebuffer(gl.FRAMEBUFFER, fbo);
            gl.clear(gl.COLOR_BUFFER_BIT | gl.DEPTH_BUFFER_BIT);
            // gl.bindFramebuffer(gl.FRAMEBUFFER, null);

            gl.useProgram(program.glShaderProgram);
            gl.viewport(0, 0, width, height);

            gl.uniform1f(program.uniforms.uInvWidth, 1 / width);
            gl.uniform1f(program.uniforms.uInvHeight, 1 / height);
            // gl.activeTexture(gl.TEXTURE0 + 0);
            // gl.bindTexture(gl.TEXTURE_2D, textures[level]);
            // gl.uniform1i(program.uniforms.uGDepth, 0);

            gl.bindBuffer(gl.ARRAY_BUFFER, this.#points);
            gl.vertexAttribPointer(program.attribs.aCoord, 2, gl.FLOAT, false, 0, 0);
            gl.enableVertexAttribArray(program.attribs.aCoord);
            gl.bindBuffer(gl.ELEMENT_ARRAY_BUFFER, this.#indices);
            gl.drawElements(gl.TRIANGLES, 6, gl.UNSIGNED_SHORT, 0);

            gl.bindFramebuffer(gl.FRAMEBUFFER, null);
        }


        // Camera pass
        for (let i = 0; i < this.meshes.length; i++) {
            // this.meshes[i].draw(this.camera, null, updatedParamters);
        }
    }
}