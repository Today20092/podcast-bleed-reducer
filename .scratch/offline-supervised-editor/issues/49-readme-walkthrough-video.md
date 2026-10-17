# Add the owner's new walkthrough to the README

Status: done
Type: task
Assignee: Codex
Blocked by: none

Owner requested adding https://youtu.be/Tumeje13o5g on 2026-10-17 after reviewing its transcript.

## Scope and acceptance

- Read YouTube metadata and English captions through yt-dlp.
- Make the newer workflow walkthrough the primary README video; retain the existing alpha demo as a secondary link.
- Describe demonstrated workflow only; preserve downloads and application behavior.
- Check links, thumbnail references, whitespace and GitHub rendering; publish through a protected PR.

## Results

- Retrieved metadata and English automatic captions with yt-dlp; the video is "Why Your Podcast Audio Sounds Bad (And How to Fix It)", 457 seconds long.
- Made its linked thumbnail the primary demo and summarized the demonstrated workflow. Kept OFSipS6v0w4 as the earlier alpha demo link.
- Passed whitespace, exact video/thumbnail link checks, thumbnail HTTP availability and a comparison confirming the downloads block is unchanged.
- Verified the new thumbnail loads through GitHub's image proxy, links to Tumeje13o5g, and the old alpha demo link still points to OFSipS6v0w4.
- Raw metadata, captions and yt-dlp's thumbnail cache stay untracked; no recording or application files changed. No local Rust tests are needed for this prose/link edit; the protected PR still requires its four hosted native checks before merging.
