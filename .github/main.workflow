workflow "Build workflow" {
  on = "push"
  resolves = ["Build", "Release"]
}

action "Build" {
  uses = "./.github/actions/build"
  args = "cargo build --release"
}

action "Release" {
  uses = "fnkr/github-action-ghr@v1"
  needs = ["Build"]
  secrets = ["GITHUB_TOKEN"]
  env = {
    GHR_PATH = "_build/"
    GHR_COMPRESS = "gz"
  }
}