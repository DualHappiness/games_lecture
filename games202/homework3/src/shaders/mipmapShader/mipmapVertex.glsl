attribute vec2 aCoord;

void main() {
    gl_Position = vec4(aCoord, 0.0, 1.0);
}