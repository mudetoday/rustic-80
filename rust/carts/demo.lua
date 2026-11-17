-- Built-in cart for the Rust core. Standard TIC-80 Lua: BOOT once, TIC every frame.

function BOOT()
  -- Tile 1: a small diamond. Nibble 0x8000 is byte 0x4000 (the BG sheet).
  for y = 0, 7 do
    for x = 0, 7 do
      local c = 0
      local d = math.abs(x - 3.5) + math.abs(y - 3.5)
      if d < 4 then c = 10 end
      if d < 2 then c = 12 end
      poke4(0x8000 + 64 + y * 8 + x, c)
    end
  end
  for x = 0, 29 do
    mset(x, 16, 1)
  end
  trace("BOOT rustic-80")
end

function TIC()
  local t = time() / 1000
  cls(0)
  for i = 0, 15 do
    rect(i * 15, 0, 15, 8, i)
  end
  local x = 20 + math.floor((math.sin(t * 2) + 1) * 80)
  local y = 30 + math.floor((math.cos(t * 1.5) + 1) * 30)
  circ(x, y, 12, 2)
  circb(x, y, 16, 4)
  rectb(8, 16, 224, 104, 6)
  spr(1, 112, 60, 0, 3, 0, 0, 1, 1)
  map(0, 16, 30, 1, 0, 120, 0, 1)
  print("RUSTIC-80", 84, 40, 12, true, 1)
  print("TIC-80 IN RUST", 70, 50, 10, true, 1)
  local bounce = math.floor(math.abs((t * 40) % 60 - 30))
  line(10, 100, 230, 100 - bounce, 14)
end
