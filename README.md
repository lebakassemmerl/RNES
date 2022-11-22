# RNES

This is a hobby-project in order to learn Rust. It tries to make use of Rust's strict type-system
and it's generics (especially the implementation of the CPU instructions). The emulator is basically
working but a lot of stuff is still missing. 

## Working
- [x] CPU
- [x] PPU but there are still a lot of improvements necessary
- [x] Controller Input
- [/] Mapers:
      + [x] Mapper 0 (NROM)
      + [x] Mapper 1 (MMC1)
      + [x] Mapper 2 (UxROM)
      + [x] Mapper 3 (CNROM)
      + [/] Mapper 4 (MMC3) partially, something is still wrong with SMB3 (I **think** it is the 
            mapper)
- [ ] APU

## Working games (not a complete list)
- Donkey Kong
- Super Mario Bros 1
- Super Mario Bros 2
- Pacman
- Zelda
- Metroid

## Building
The only dependency to build this project is to install sdl2 (and of course rust). E.g. on Archlinux
you can install sdl2 with the following command:
```bash
sudo pacman -S sdl2
```

In order to build the project simply run:
```bash
cargo build
```

When you want to play a game smooth, build the project with the release option, otherwise it is
extremely slow (has to be fixed):
```bash
cargo run --release <path to rom>
```

## TODO
A todo-list can be found in doc/todo.md which contains a lot of stuff which as to be implemented,
fixed or improved.
