# Publish Checklist

## Setup `tod-bin`

Create `tod-bin` directory for pushing to AUR

```fish
./scripts/setup_aur.sh
```

## Publish Procedure

1. Update `CHANGELOG.md` with version number
2. Update the version number in this file
3. Create PR with

```fish
VERSION=0.7.5 ./scripts/create_pr.sh
```

4. Wait for it to pass, then merge and pull in latest changes

```fish
merge
```

5. Release it to all the places

```fish
VERSION=0.7.5 NAME=tod ./scripts/release.sh
```

