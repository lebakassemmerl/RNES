# TODO / To Fix:

## CPU
- [ ] Check if the IRQ from the mappers returns the correct cycles and does the correct thing

## PPU: 
- [ ] Sprite evaluation during visible cycles and not at the end
- [ ] Consider the half-frame (odd frame) when background rendering is active
- [ ] Correct writes to register 0x2007
- [ ] Handle reading PPUPDATA (0x2004) during visible scanlines
- [ ] Consider Bits 1 & 2 in PPUMASK register
- [ ] Color emphasis

## Mapper
- [/] Fix MMC3 -> currently kind of working
      + [x] Make ROMs startable/playable or at least display something without crash
      + [ ] Fix weird statusbar issue in SMB3 (low bar is always f*cked up)
      + [ ] Fix vertical(?) scrolling issue (SMB3 in 1st castle). Not sure if this is really a bug
            in the MMC3 mapper implementation.

- [x] Fix MMC1
      + [x] Make ROMs startable/playable or at least display something without crash
      + [x] Scrolling issue -> confused vertical and horizontal. Zelda and Metroid are now working
            correctly

## Misc
- [ ] Stabilize to 60Hz
- [ ] Fix weird SDL2 bug that sometimes the background goes black
- [ ] Custom keymapping
- [ ] Controller 2 Support

## ???
- [ ] weird bug on iceclimbers where the CPU tries to read from address 0x589a
- [ ] Bug with SMB3 and SMB2 where the sprices look a bit wrong and too transparent. Not sure if
      it is related to MMC3 implementation (which is not completely correct) or if it is caused by
      an incorrect sprite-evaluation (priority!)