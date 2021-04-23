class WebGLRenderer {
    meshes = [];
    shadowMeshes = [];
    lights = [];
    index = 0;
    step = Math.PI / 1000;


    constructor(gl, camera) {
        this.gl = gl;
        this.camera = camera;
    }

    addLight(light) {
        this.lights.push({
            entity: light,
            meshRender: new MeshRender(this.gl, light.mesh, light.mat)
        });
    }
    addMeshRender(mesh) { this.meshes.push(mesh); }
    addShadowMeshRender(mesh) { this.shadowMeshes.push(mesh); }

    render() {
        const gl = this.gl;

        gl.clearColor(0.0, 0.0, 0.0, 1.0); // Clear to black, fully opaque
        gl.clearDepth(1.0); // Clear everything
        gl.enable(gl.DEPTH_TEST); // Enable depth testing
        gl.depthFunc(gl.LEQUAL); // Near things obscure far things


        console.assert(this.lights.length != 0, "No light");
        // console.assert(this.lights.length == 1, "Multiple lights");

        {// move 
            let offset = Math.max(0.5, 2 * Math.sin(this.index));
            this.index += this.step;
            let scale = [offset, offset, offset];
            for (let i = 0; i < this.shadowMeshes.length; i++) {
                let mesh = this.shadowMeshes[i].mesh;
                vec3.mul(mesh.transform.scale, mesh.originTransform.scale, scale);
            }
        }


        for (let l = 0; l < this.lights.length; l++) {
            // Draw light
            // TODO: Support all kinds of transform
            let light = this.lights[l];
            light.meshRender.mesh.transform.translate = light.entity.lightPos;
            light.meshRender.draw(this.camera);

            if (light.entity.hasShadowMap == true) {
                gl.bindFramebuffer(gl.FRAMEBUFFER, light.entity.fbo);
                gl.clear(gl.DEPTH_BUFFER_BIT | gl.COLOR_BUFFER_BIT);
                // gl.clearColor(0.0, 0.0, 0.0, 1.0);
                // gl.clearDepth(1.0);
            }
        }

        // Shadow pass
        for (let i = 0; i < this.shadowMeshes.length; i++) {
            let meshRender = this.shadowMeshes[i];
            meshRender.material.updateLightMVP(meshRender.mesh.transform.translate, meshRender.mesh.transform.scale);
            meshRender.draw(this.camera);
        }

        let lights = this.lights.map(l => l.entity);
        // Camera pass
        for (let i = 0; i < this.meshes.length; i++) {
            this.gl.useProgram(this.meshes[i].shader.program.glShaderProgram);
            let lightPosArr = new Float32Array(this.lights.length * 3);
            for (let l = 0; l < this.lights.length; l++) {
                lightPosArr[3 * l + 0] = this.lights[l].entity.lightPos[0];
                lightPosArr[3 * l + 1] = this.lights[l].entity.lightPos[1];
                lightPosArr[3 * l + 2] = this.lights[l].entity.lightPos[2];
            }
            this.gl.uniform3fv(this.meshes[i].shader.program.uniforms.uLightPos, lightPosArr);
            this.meshes[i].material.updateLightMVP(lights, this.meshes[i].mesh.transform.translate, this.meshes[i].mesh.transform.scale);
            this.meshes[i].draw(this.camera);
        }
    }
}