class WebGLRenderer {
    meshes = [];
    shadowMeshes = [];
    lights = [];

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

        for (let l = 0; l < this.lights.length; l++) {
            // Draw light
            // TODO: Support all kinds of transform
            this.lights[l].meshRender.mesh.transform.translate = this.lights[l].entity.lightPos;
            this.lights[l].meshRender.draw(this.camera);

            // Shadow pass
            if (this.lights[l].entity.hasShadowMap == true) {
                for (let i = 0; i < this.shadowMeshes.length; i++) {
                    this.shadowMeshes[i].draw(this.camera);
                }
            }


        }
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
            this.meshes[i].draw(this.camera);
        }

    }
}