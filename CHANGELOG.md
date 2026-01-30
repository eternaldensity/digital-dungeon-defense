# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

## [Unreleased]

### Added
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
- Fix speed modifier applying every frame (#21)

### Changed
- Add win condition when all enemies in wave defeated (#20)
- Add lose condition when core is destroyed (#19)
- Add GameState resource tracking core health and wave status (#18)
- Add system to spawn enemies based on wave config (#16)
- Add wave state machine (Preparing, Active, Complete) (#15)
- Add WaveConfig resource with enemy counts and timing (#14)
- Add system to fire DamageEvent when attack lands (#12)
- Add system to detect enemies in attack range (#11)
- Add attack cooldown and timing to CombatStats (#10)
