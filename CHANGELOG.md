# Deeb Changelog

## v0.0.11

- Add Count, Skip, Limit, and Order functionatlity.

## v0.0.10

- Improve ACID Compliance with fsync.

## v0.0.9

### Fixed

- Custom error type to prevent need to install anyhow.

## v0.0.8

### Added

- Improved DX with Proc Macro Support.

## v0.0.7

### Fixed

- Update Readme and Republish

## v0.0.6

### Added

- **BREAKING** Typed Inserts and Updates - Inserts and Updates now require a struct which implements DeserializedOwned or Serialize to provide safer handling of typed data.

### Fixed

- FS Warning Resolved

## v0.0.4 

### Added

- Associate entities, primary keys, and joins.
- Drop key and add key functionality.

### Changed

- Improved array support.

- Defining an entity now has a `new` impl.

## v0.0.3

### Added

- Drop Key - Removes a key from every entity, if exists.
- Add Key - Add a key with a default value to every entity.

## v0.0.2

### Changed

- Update the Query object to implement `into` to clean up query creations.
- Update documentation.

## v0.0.1

Release includes basic operations and transactions.
