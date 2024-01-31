from typing import Any
import pygame
from pygame.locals import *
from spritesheet import SpriteSheet
import sys
from random import choice

pygame.init()

DISPLAYSURF = pygame.display.set_mode((640, 480), DOUBLEBUF)

pygame.display.set_caption("Map Rendering Demo")
FPSCLOCK = pygame.time.Clock()

# map_data = [
#     [1, 1, 1, 1, 1],
#     [1, 0, 0, 0, 1],
#     [1, 0, 0, 0, 1],
#     [1, 0, 0, 0, 1],
#     [1, 0, 0, 0, 1],
#     [1, 1, 1, 1, 1],
# ]

ORIENTATION = {
    "NE": 0,
    "NW": 1,
    "SW": 2,
    "SE": 3,
}

class Terrain:

    def __init__(self, sprite_sheet, map_tiles, size_x, size_y):
        self.sprite_sheet = sprite_sheet
        self.map_tiles = map_tiles
        self.size_x = size_x
        self.size_y = size_y

    def get_image(self, x, y):
        start_x = x * self.size_x
        start_y = y * self.size_y
        return self.sprite_sheet.image_at((start_x, start_y, self.size_x, self.size_y), -1)

    def image_for(self, tile_type):
        if tile_type == 0:
            return self.get_image(choice(range(0, 10)), 0)
        elif tile_type == 1:
            return self.get_image(choice(range(0, 10)), 19)


map_data = [
    [1] * 10,
    [0] * 10,
    [1] * 10,
    [1, 1, 1, 1],
]

filename = "map.png"
terrain_sheet = SpriteSheet(filename)

TILEWIDTH = 64
TILEHEIGHT = 64
TILEHEIGHT_HALF = TILEHEIGHT / 2
TILEWIDTH_HALF = TILEWIDTH / 2

terrain = Terrain(terrain_sheet, map_data, TILEWIDTH, TILEHEIGHT_HALF)

terrain_rect = (0, 0, TILEWIDTH, TILEHEIGHT_HALF)
terrain_image = terrain_sheet.image_at(terrain_rect, -1)

for row_nb, row in enumerate(map_data):
    for col_nb, tile in enumerate(row):
        tileImage = terrain.image_for(tile)
        cart_x = row_nb * TILEWIDTH_HALF
        cart_y = col_nb * TILEHEIGHT_HALF
        iso_x = cart_x - cart_y
        iso_y = (cart_x + cart_y) / 2
        centered_x = DISPLAYSURF.get_rect().centerx + iso_x
        centered_y = DISPLAYSURF.get_rect().centery / 2 + iso_y
        DISPLAYSURF.blit(tileImage, (centered_x, centered_y))  # display the actual tile

while True:
    for event in pygame.event.get():
        if event.type == QUIT:
            pygame.quit()
            sys.exit()
        if event.type == KEYUP:
            if event.key == K_ESCAPE:
                pygame.quit()
                sys.exit()

    pygame.display.flip()
    FPSCLOCK.tick(30)