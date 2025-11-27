# -*- coding: utf-8 -*-
"""
SikuliX サンプルスクリプト / SikuliX Sample Script
マウスカーソルを画面の四隅を行き来させる
Mouse cursor moves between four corners of the screen
"""
import time

# 画面サイズを取得 / Get screen dimensions
screen = Screen()
w = screen.getW()
h = screen.getH()

print(f"Screen size: {w} x {h}")

# 四隅の座標 / Coordinates of four corners
corners = [
    (0, 0),           # 左上 / Top-left
    (w - 1, 0),       # 右上 / Top-right
    (w - 1, h - 1),   # 右下 / Bottom-right
    (0, h - 1),       # 左下 / Bottom-left
]

# 繰り返し回数 / Number of repetitions
repeat = 3

print("Starting mouse movement...")

for i in range(repeat):
    print(f"Round {i + 1}/{repeat}")
    for idx, (x, y) in enumerate(corners):
        corner_names = ["Top-left", "Top-right", "Bottom-right", "Bottom-left"]
        print(f"  Moving to {corner_names[idx]} ({x}, {y})")
        mouseMove(Location(x, y))
        time.sleep(0.5)  # 0.5秒待機 / Wait 0.5 seconds

print("Done! / 完了!")
