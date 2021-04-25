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
        console.assert(this.lights.length == 1, "Multiple lights");

        const timer = Date.now() * 0.0001;

        for (let l = 0; l < this.lights.length; l++) {
            // Draw light
            this.lights[l].meshRender.mesh.transform.translate = this.lights[l].entity.lightPos;
            this.lights[l].meshRender.draw(this.camera);

            // Camera pass
            for (let i = 0; i < this.meshes.length; i++) {
                this.gl.useProgram(this.meshes[i].shader.program.glShaderProgram);
                this.gl.uniform3fv(this.meshes[i].shader.program.uniforms.uLightPos, this.lights[l].entity.lightPos);

                for (let k in this.meshes[i].material.uniforms) {

                    let cameraModelMatrix = mat4.create();
                    //mat4.fromRotation(cameraModelMatrix, timer, [0, 1, 0]);

                    if (k == 'uMoveWithCamera') { // The rotation of the skybox
                        gl.uniformMatrix4fv(
                            this.meshes[i].shader.program.uniforms[k],
                            false,
                            cameraModelMatrix);
                    }

                    if (k.startsWith('uPrecomputeL')) {
                        let precomputeL_RGBMat3 = precomputeL[PRT_TYPES[guiParams.prtType]][guiParams.envmapId];
                        let arr = getMat3ValueFromRGB(precomputeL_RGBMat3);
                        for (let idx = 0; idx < 3; idx++) {
                            this.gl.uniformMatrix3fv(this.meshes[i].shader.program.uniforms[`uPrecomputeL[${idx}]`], false, arr[idx]);
                        }
                    }
                    // Bonus - Fast Spherical Harmonic Rotation
                    //let precomputeL_RGBMat3 = getRotationPrecomputeL(precomputeL[guiParams.envmapId], cameraModelMatrix);

                }

                this.meshes[i].draw(this.camera);
            }
        }

    }
}