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
    vec4 col =   texture(inputTexture, fragTexCoord);
    float a = col.a; 
    int width =5;
    for(int i  = -width; i<width+1; i++){
        for(int j = -width; j<width+1; j++){
            if(i == 0 && j == 0){
                continue;
            }
            vec4 c2 = texture(inputTexture, fragTexCoord+vec2(i,j));
            if(c2.r >= 0.6|| c2.g >= 0.6|| c2.b >=0.6){
                   col += c2/sqrt((i*i+j*j)/32.0);
            }
        }
    }
    finalColor =col;
    finalColor.a = a;
}
