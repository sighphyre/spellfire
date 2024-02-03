from typing import Any
import pygame
from pygame.locals import *
from spritesheet import SpriteSheet
import sys
from random import choice
import pytmx
from pytmx import TiledImageLayer, TiledObjectGroup, TiledTileLayer
from pytmx.util_pygame import load_pygame

pygame.init()

DISPLAYSURF = pygame.display.set_mode((1280, 900), pygame.RESIZABLE)

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
        return self.sprite_sheet.image_at(
            (start_x, start_y, self.size_x, self.size_y), -1
        )

    def image_for(self, tile_type):
        if tile_type == 0:
            return self.get_image(1, 0)
        elif tile_type == 1:
            return self.get_image(1, 19)
        elif tile_type == 2:
            return self.get_image(3, 1)


class ViewPort:

    def __init__(self, init_x, init_y):
        self.x = init_x
        self.y = init_y

    def apply_offset(self, x=0, y=0):
        self.x += x
        self.y += y


class TiledRenderer:

    def __init__(self, filename) -> None:
        tm = load_pygame(filename)
        self.pixel_size = tm.width * tm.tilewidth, tm.height * tm.tileheight
        self.tmx_data = tm

    def render_map(self, surface) -> None:
        if self.tmx_data.background_color:
            surface.fill(pygame.Color(self.tmx_data.background_color))

        for layer in self.tmx_data.visible_layers:

            if isinstance(layer, TiledTileLayer):
                self.render_tile_layer(surface, layer)

            elif isinstance(layer, TiledObjectGroup):
                self.render_object_layer(surface, layer)

            elif isinstance(layer, TiledImageLayer):
                self.render_image_layer(surface, layer)

    def render_tile_layer(self, surface, layer) -> None:
        tw = self.tmx_data.tilewidth
        th = self.tmx_data.tileheight
        surface_blit = surface.blit

        if self.tmx_data.orientation == "orthogonal":
            for x, y, image in layer.tiles():
                surface_blit(image, (x * tw, y * th))
        elif self.tmx_data.orientation == "isometric":
            ox = self.pixel_size[0] // 2
            tw2 = tw // 2
            th2 = th // 2
            for x, y, image in layer.tiles():
                sx = x * tw2 - y * tw2
                sy = x * th2 + y * th2
                surface_blit(image, (sx + ox, sy))

    def render_object_layer(self, surface, layer) -> None:
        """Render all TiledObjects contained in this layer"""
        draw_lines = pygame.draw.lines
        surface_blit = surface.blit

        rect_color = (255, 0, 0)
        for obj in layer:

            if obj.image:
                surface_blit(obj.image, (obj.x, obj.y))

            else:
                draw_lines(
                    surface, rect_color, obj.closed, obj.apply_transformations(), 3
                )

    def render_image_layer(self, surface, layer) -> None:
        if layer.image:
            surface.blit(layer.image, (0, 0))


filename = "map.png"
terrain_sheet = SpriteSheet(filename)

TILEWIDTH = 64
TILEHEIGHT = 64
TILEHEIGHT_HALF = TILEHEIGHT / 2
TILEWIDTH_HALF = TILEWIDTH / 2

map_data = [[1, 1, 1]]

terrain = Terrain(terrain_sheet, map_data, TILEWIDTH, TILEHEIGHT_HALF)
viewport = ViewPort(0, 0)


class Game:
    def __init__(self, render_target):
        self.renderer = TiledRenderer("tiled/map.tmx")
        self.render_target = render_target

    def _draw(self, surface):
        temp = pygame.Surface(self.renderer.pixel_size)

        self.renderer.render_map(temp)
        pygame.transform.smoothscale(temp, surface.get_size(), surface)

        # display a bit of use info on the display
        f = pygame.font.Font(pygame.font.get_default_font(), 20)
        i = f.render("press any key for next map or ESC to quit", 1, (180, 180, 0))
        surface.blit(i, (0, 0))

        pygame.display.flip()

    def run(self):
        self.dirty = True
        self.running = True
        self.exit_status = 1

        while self.running:
            # self.handle_input()

            if self.dirty:
                self._draw(self.render_target)
                self.dirty = False
                pygame.display.flip()

        return self.exit_status

    def handle_input(self) -> None:
        try:
            event = pygame.event.wait()

            if event.type == QUIT:
                self.exit_status = 0
                self.running = False

            elif event.type == KEYDOWN:
                if event.key == K_ESCAPE:
                    self.exit_status = 0
                    self.running = False
                else:
                    self.running = False

            elif event.type == VIDEORESIZE:
                pygame.display.set_mode((event.w, event.h), pygame.RESIZABLE)
                self.dirty = True

        except KeyboardInterrupt:
            self.exit_status = 0
            self.running = False


game = Game(DISPLAYSURF)
game.run()
