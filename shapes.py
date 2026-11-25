import math
from PIL import Image
from PIL import ImageFont
from PIL import ImageDraw

# ----------------- começo da geração por IA -----------------
import numpy as np

def gerar_pontos_circulo(num_pontos, raio=1.0, centro_x=0.0, centro_y=0.0):
    # Gerar distribuições uniformes entre 0 e 1
    u = np.random.uniform(0, 1, num_pontos)
    v = np.random.uniform(0, 1, num_pontos)
    
    # Aplicar a transformação matemática
    theta = 2 * np.pi * u
    r = raio * np.sqrt(v)  # A raiz quadrada garante a uniformidade espacial
    
    # Converter para coordenadas cartesianas e ajustar o centro
    x = centro_x + r * np.cos(theta)
    y = centro_y + r * np.sin(theta)
    
    return x, y

# Exemplo: gerar 1000 pontos em um círculo de raio 5 centrado em (0,0)
x_pontos, y_pontos = gerar_pontos_circulo(1000, raio=5.0)
# ----------------- fim da geração por IA -----------------
 
def lightness(pixel, negative = False):
    r = pixel[0]
    g = pixel[1]
    b = pixel[2]
    result = (r*0.2126+g*0.7152+b*0.0722)/255
    if negative:
        result = 1 - result
    return result

def draw_samples_circles(h_samples,v_samples, subsamples):
    global size
    overlay  = Image.new(mode = "RGBA", size = size)
    draw = ImageDraw.Draw(overlay)
    
    v_sample_size = size[1]/v_samples
    h_sample_size = size[0]/h_samples

    offset_y = v_sample_size/2;
    offset_x = h_sample_size/2;
    r = v_sample_size/2 if v_sample_size > h_sample_size else h_sample_size/2
    r *= 1.3

    collumn_offset_y = offset_y/2
    for i in range(v_samples):
        for j in range(h_samples):
            coy = collumn_offset_y * (j%2)
            draw.ellipse(
                [
                    j*h_sample_size+offset_x - r,
                    i*v_sample_size+offset_y - r + coy,
                    j*h_sample_size+offset_x + r,
                    i*v_sample_size+offset_y + r + coy,
                ],
                fill = (255,0,0,100), outline = (255,255,255,100), width = 3
            )
            
    # overlay = overlay.convert("RGB")
    overlay.save("circles.png")
    overlay.show()


def create_temp_image(char,size):
    size = (200,200)
    img  = Image.new(mode = "RGB", size = size)
    draw = ImageDraw.Draw(img)
    font = ImageFont.truetype(font_path,min(size[0],size[1]))
    draw.text((0,0), char,(255,255,255), font= font)

    return img

    
def get_char_shape_vector(char,size, h_samples,v_samples, subsamples):
    img = create_temp_image(char,size)
    imgpx = img.load()
    v_sample_size = img.size[1]/v_samples
    h_sample_size = img.size[0]/h_samples

    offset_y = v_sample_size/2;
    offset_x = h_sample_size/2;
    r = v_sample_size/2 if v_sample_size > h_sample_size else h_sample_size/2
    r *= 1.3

    collumn_offset_y = offset_y/2

    sample_points_x, sample_points_y = gerar_pontos_circulo(subsamples, r)

    shape = []
    for i in range(v_samples):
        for j in range(h_samples):
            value = 0
            coy = collumn_offset_y * (j%2)
            x = j*h_sample_size+offset_x
            y = i*v_sample_size+offset_y + coy
            for k in range(subsamples):
                nx = math.floor(x+sample_points_x[k])
                ny = math.floor(y+sample_points_y[k])
                if nx >= img.size[0] or ny>=img.size[1] or nx < 0 or ny< 0 :
                    continue
                value += 1*(imgpx[nx,ny][0] == 255) #lightness(imgpx[nx,ny])
            value /= subsamples
            shape.append(value)
    return shape

hsamples = 3
vsamples = 3
font_path = "TrmnlCubic-Regular.ttf"
max_values = [0 for i in range(hsamples*vsamples)]
allshapes = []
for i in range(32,127):
    result = get_char_shape_vector(chr(i), 200, hsamples, vsamples, 100)
    allshapes.append(result)
    for j in range(hsamples*vsamples):
        max_values[j] = max(result[j],max_values[j])

for s in range(len(allshapes)):
    for i in range(len(max_values)):
        allshapes[s][i] = allshapes[s][i]/max_values[i]

with open("shapes.txt","w") as file:
    file.write(str(allshapes))
    file.close()

