attribute vec2 aCoord;
uniform float uInvWidth;
uniform float uInvHeight;

void main() {
    gl_Position = vec4(aCoord, 0.0, 1.0);
}