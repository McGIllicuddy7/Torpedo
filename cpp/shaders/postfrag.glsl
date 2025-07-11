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
    vec4 col0 = col;
    col = tan(col*1.5);
    /*
    if(col.r<col0.r){col.r = col0.r;}
    if(col.g<col0.g){col.g = col0.g;}
    if(col.b<col0.b){col.b = col0.b;}*/
    float a = col.a; 
    float total = 0.0;
    int width =5;
    for(int i  = -width; i<width+1; i++){
        for(int j = -width; j<width+1; j++){
            if(i == 0 && j == 0){
                continue;
            }
            vec4 c2 = texture(inputTexture, fragTexCoord+vec2(i,j));
            float div = i*i+j*j;
            total += 1/div;
            float thresh = 1000.0;
            if(c2.r >= thresh){ 
                   col.r += c2.r/div;
            }
            if(c2.g >= thresh){
                col.g += c2.g/div;
            }
            if(c2.b>= thresh){
                col.b += c2.b/div;
            }
        }
    }
    col/=total*3.0;
    if(col.r>1){col.r = 1;}
    if(col.g>1){col.g = 1;}
    if(col.b>1){col.b = 1;}
    finalColor = atan(col*1.5);
    finalColor.a = 1.0;
}
