#version 330

// Input vertex attributes (from vertex shader)
in vec2 fragTexCoord;
in vec4 fragColor;
in vec3 fragPosition;
in vec3 fragNormal;

// Input uniform values
uniform sampler2D texture0;
uniform vec4 colDiffuse;

// Output fragment color
out vec4 finalColor;

// NOTE: Add your custom variables here
uniform vec3 light_positions[16];
uniform vec4 light_colors[16];
uniform int light_count;

uniform vec4 directional_light_color;
uniform vec3 camera_position;
uniform vec4 ambient;
uniform vec3 directional_light_direction;
uniform sampler2D lightmap;
vec4 contribution(int light_idx, vec4 texel_color) {
  vec4 ot = vec4(0.0, 0.0, 0.0, 0.0);
  vec3 delta = light_positions[light_idx] - fragPosition;
  float len = length(delta);
  if (len > 0.0) {
    vec3 dt = normalize(delta);
    ot = light_colors[light_idx] * (dot(dt, fragNormal)) / (len) * 10;
  }
  ot.r *= texel_color.r;
  ot.g *= texel_color.g;
  ot.b *= texel_color.b;
  ot.a *= texel_color.a;
  return ot;
}

void main() {
  vec4 texelColor =
      texture(lightmap, fragTexCoord); // texture(texture0, fragTexCoord);
  vec4 tmp = ambient + dot(directional_light_direction, fragNormal) *
                           directional_light_color;
  tmp.r *= texelColor.r;
  tmp.g *= texelColor.g;
  tmp.b *= texelColor.b;
  for (int i = 0; i < light_count; i++) {
    tmp += contribution(i, texelColor);
  }
  tmp.r = pow(tmp.r, 1. / 1.1);
  tmp.g = pow(tmp.g, 1. / 1.1);
  tmp.b = pow(tmp.b, 1. / 1.1);
  tmp.a = texelColor.a;
  finalColor = tmp;
}
