#version 330 core

in vec2 v_uv;

uniform float u_time;
uniform vec2 u_resolution;

out vec4 frag_color;

void main() {
    vec2 uv = v_uv;
    float aspect = u_resolution.x / max(u_resolution.y, 1.0);
    uv.x *= aspect;

    float wave1 = sin(uv.x * 2.2 + u_time * 0.35);
    float wave2 = cos(uv.y * 3.0 - u_time * 0.25);
    float glow = 0.5 + 0.5 * sin(uv.x * 1.7 + uv.y * 1.3 + u_time * 0.2);

    vec3 base = vec3(0.025, 0.035, 0.065);
    vec3 accent = vec3(0.12, 0.32, 0.48);
    vec3 color = mix(base, accent, glow * 0.35);
    color += vec3(0.025, 0.04, 0.07) * (wave1 * wave2 + 1.0);

    frag_color = vec4(color, 1.0);
}
