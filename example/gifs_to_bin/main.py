import sys

from PIL import Image, ImageSequence
import numpy as np

 
def image_to_u16_big_endian_bytes(img, output_size) -> tuple[np.ndarray,int]:

    img           = img.convert("RGB")
    width, height = img.size
    min_dim       = min(width, height)
    img           = img.crop((0, 0, min_dim, min_dim))

    if output_size != -1:
        img = img.resize((output_size, output_size), Image.Resampling.LANCZOS)

    img_16 = np.array(img, dtype=np.uint16) * 257
    # (8-bit 0–255 scaled to 16-bit 0–65535)

    # pack into single 16-bit per pixel (RGB565 format)
    r = (img_16[:, :, 0] >> 11) & 0x1F
    g = (img_16[:, :, 1] >> 10) & 0x3F
    b = (img_16[:, :, 2] >> 11) & 0x1F

    rgb565 = (r << 11) | (g << 5) | b
    return ( rgb565.byteswap(), min_dim)

def jpg_to_bytes(path_img, outfile, output_size):
    rgb565_array, final_size = image_to_u16_big_endian_bytes(Image.open(path_img), output_size)
    rgb565_array.tofile(f"{outfile}_{output_size}x{final_size}.bin")


def gif_to_bytes(path_gif, outfile, output_size):
    byte_arrays = []
    final_size  = None
    with Image.open(path_gif) as im:
    # Iterate through every frame in the GIF
        for frame in ImageSequence.Iterator(im):
            
            rgb565_array, final_size = image_to_u16_big_endian_bytes(frame, output_size)
            flattened_byte_array     = rgb565_array.flatten()
            byte_arrays.append(flattened_byte_array)
    

    frame_count = len(byte_arrays)
    gif_bytes   = np.concatenate(byte_arrays)
    gif_bytes.tofile(f"{outfile}_{final_size}x{final_size}_frames={frame_count}.bin")

    


if __name__ == "__main__":
    if len(sys.argv) < 4:
        print("Usage: main.py \"mode\" \"input\" \"output\" size(optional)")
        print("modes: JPG_TO_BIN | GIF_TO_BIN")
        print("Example: uv run main.py GIF_TO_BIN \"nyan.GIF\" \"nyan_binary\" 240 --> nyan_binary_240x240_frames=17.bin")
        sys.exit(0) 

    input_name = sys.argv[2]
    output_name = sys.argv[3]
    output_size = -1

    if len(sys.argv) == 5:
        output_size = sys.argv[4]
    match sys.argv[1]:
        case "JPG_TO_BIN":
            jpg_to_bytes(input_name, output_name, int(output_size))
        case "GIF_TO_BIN":
            gif_to_bytes(input_name, output_name, int(output_size))
                        
