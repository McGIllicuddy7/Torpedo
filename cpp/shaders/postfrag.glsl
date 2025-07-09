#version 330
in vec2 fragTexCoord;
in vec4 fragColor;
in vec3 fragNormal;
in vec3 fragPosition;
// Input uniform values
uniform sampler2D inputTexture;
//uniform sampler2D normals;
//uniform vec4 colDiffuse;
// Output fragment color
out vec4 finalColor;
void main(){
    finalColor = texture(inputTexture, fragTexCoord);
}
