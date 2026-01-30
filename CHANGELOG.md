# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
- Add WASD and numpad controls for camera panning and tile selection (#44)
- Add mousewheel zoom control for camera (#38)
- Add arrow key controls to pan camera target location (#37)
- Convert rendering from 2D sprites to 3D meshes (#30)
- Design and implement integrated gameplay loop with title screen and building mode (#28)
- Design building mode and tile-based floor planning UI (#24)
- Add floor damage modifier application to combat (#22)
- Add win/lose conditions for waves (#17)
- Add wave spawning system for enemy progression (#13)
- Add auto-attack system for creatures (#9)
- Add resource collection visuals (#7)
- Add combat/damage visual feedback (#6)
- Add multi-floor rendering (#5)
- Add floor type visual differentiation (#4)
- Add core pulsing animation (#3)
- Add spawner visuals (#2)
- Add trap visuals (Firewall, DataSpike) (#1)

### Fixed
- Fix tile grid boundary issues and implement level-based sizing (#39)
- Fix duplicate grid rendering - old grid drawn alongside new tile grid (#29)
- Fix speed modifier applying every frame (#21)

### Changed
- Add 2D menu screen with title and start instructions (#45)
- Constrain camera pan to current level boundaries (#43)
- Implement level-based grid sizing (3x3 base, +2 every 2 levels) (#42)
- Fix grid border lines showing partial tiles (#41)
- Fix tiles being placed beyond grid extent (#40)
- Update camera to use perspective projection (#31)
- Add visual rendering for planning tiles in build mode (#27)
- Add building mode input handling for tile placement (#26)
- Create crates/building/ with core data structures (#25)
- Analyze and prioritize missing gameplay features from design doc (#23)
- Add win condition when all enemies in wave defeated (#20)
- Add lose condition when core is destroyed (#19)
- Add GameState resource tracking core health and wave status (#18)
- Add system to spawn enemies based on wave config (#16)
- Add wave state machine (Preparing, Active, Complete) (#15)
- Add WaveConfig resource with enemy counts and timing (#14)
- Add system to fire DamageEvent when attack lands (#12)
- Add system to detect enemies in attack range (#11)
- Add attack cooldown and timing to CombatStats (#10)
