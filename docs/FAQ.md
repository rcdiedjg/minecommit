# Frequently Asked Questions (FAQ)

Here are some frequently asked questions that may help answer your queries.

- [Can Git be used this way?](#can-git-be-used-this-way)
- [How is it different from other backup tools? What other features does it have besides high performance?](#how-is-it-different-from-other-backup-tools-what-other-features-does-it-have-besides-high-performance)
- [Is there a Mod?](#is-there-a-mod)
- [Does it support scheduled backups?](#does-it-support-scheduled-backups)
- [Does it support non-vanilla games? What about Mods/Plugins?](#does-it-support-non-vanilla-games-what-about-modsplugins)
- [Is it suitable for servers?](#is-it-suitable-for-servers)
- [Is there a limit on save size?](#is-there-a-limit-on-save-size)
- [Can I upload saves to GitHub?](#can-i-upload-saves-to-github)
- [Other questions?](#other-questions)

## Can Git be used this way?

**Yes**. Although some advanced Git operations (like `git-diff` `git-blame`) cannot work on binary files the way they do on text files, Git has a very powerful incremental compression mechanism at the storage layer. When you only modify a small amount of data in your save, `git-pack-objects` uses a byte-based differential algorithm (based on `xdelta`). It can efficiently identify the similarities between two binary versions and only store the differences.

We use a specially tuned decompression mechanism to efficiently convert high-entropy raw files into a diff-friendly format, allowing Git to perfectly capture and compress redundant parts between files.

Additionally, we are planning to implement features like `minecommit diff` to enable block-level operations on saves.

## How is it different from other backup tools? What other features does it have besides high performance?

First, we have significant differences in our backup implementation:

- We use Git's built-in differential algorithm for deduplication, with each backup taking only ~1% of the save space
- Most backup tools still use `.zip` format for full compression of each backup, requiring ~80% of the save space per backup
- Some incremental backup tools (like [QuickBackupM](https://github.com/QuickBackupMultiMod-Dev/QuickBackupM-Reforged) [PrimeBackup](https://github.com/TISUnion/PrimeBackup) [MineBackup](https://github.com/Leafuke/MineBackup)) apply incremental compression algorithms at different levels, but none implement decompression for high-entropy save files, so there's still room for improvement in single incremental backup sizes.

> _TODO: Add performance comparison table_

Secondly, we leverage Git's best practices for version control, which means you can apply almost any Git tool (like VSCode or GitHub) to your save's Git repository — something other products cannot do.

## Is there a Mod?

**Not currently**, for two reasons:

1. We don't want to add too many non-core features in the early stages of the project. We want to focus time and energy on core functionality development.
2. We don't have Mod development experience.

If you're a regular user, please use the [GUI program](../README.md#using-the-gui). If you're a Mod developer, feel free to submit a Pull Request! <3

## Does it support scheduled backups?

The GUI does not have built-in scheduled backup functionality. You can use the CLI program together with scheduled task tools (like [crontab](https://linux.die.net/man/5/crontab) or [PowerShell ScheduledTasks](https://learn.microsoft.com/en-us/powershell/module/scheduledtasks/?view=windowsserver2025-ps)) to achieve this functionality. As Malcolm McIlroy, inventor of the Unix pipe mechanism, said:

> Programs should do one thing and do it well.

## Does it support non-vanilla games? What about Mods/Plugins?

**In most cases, yes.** MineCommit uses glob expressions to capture files and parses them based on the vanilla game's data persistence logic. At the same time, we are also working on compatibility with mainstream modpacks to achieve better performance and provide users with an improved experience.

Nevertheless, if you encounter logs like the following during runtime:

```log
[2026-05-31T05:20:27Z ERROR minecommit] Skipped file: SDMEconomy/1fe4a2e6-f3eb-3fad-82b8-e846006e55ad.data
[2026-05-31T05:20:27Z ERROR minecommit] Skipped file: SDMEconomy/7f200e67-eda0-3ae6-a586-324c5973c8b2.data
[2026-05-31T05:20:27Z ERROR minecommit] Skipped file: SDMEconomy/c46dafc2-ec52-3dc1-a16b-3da31a5d6be7.data
[2026-05-31T05:20:27Z ERROR minecommit] Skipped file: playerdata/1fe4a2e6-f3eb-3fad-82b8-e846006e55ad.cosarmor
[2026-05-31T05:20:27Z ERROR minecommit] Skipped file: playerdata/7f200e67-eda0-3ae6-a586-324c5973c8b2.cosarmor
[2026-05-31T05:20:27Z ERROR minecommit] Skipped file: playerdata/c46dafc2-ec52-3dc1-a16b-3da31a5d6be7.cosarmor
Error: Skipped 6 files because they are not caught by any handler. Catch them via -p or ignore them via -i.
```

This means some mods require manual configuration. For the above case, we can add the `-p "SDMEconomy/*.data" -p "playerdata/*.cosarmor"` parameters to capture them.

Additionally, in very rare cases, some mods may modify the vanilla data persistence logic. If you encounter this issue, feel free to submit an issue report.

## Is it suitable for servers?

**Yes**, for Paper, Fabric, and other server cores that haven't modified the vanilla storage mechanism. For other server cores and server plugins, please refer to the previous FAQ.

If you can log in to the server's backend terminal (note: this is not the game server core's console), you can directly download and run the CLI program. If you cannot log in to the server's backend terminal but can access the in-game console, you can install [ConsoleMC](https://modrinth.com/mod/consolemc) or a similar plugin/Mod to run host programs from the console. We will add more server-side support in the future.

## Is there a limit on save size?

There is **no limit**, but you should keep your save size under 25% of your total hard drive space. Otherwise, migrating to a larger hard drive or deleting Git history can be troublesome later.

## Can I upload saves to GitHub?

**Theoretically yes**, as long as your repository meets [GitHub's requirements](https://docs.github.com/en/repositories/creating-and-managing-repositories/repository-limits).

If your repository doesn't meet the hosting platform's requirements, we recommend deploying Gitea or similar tools yourself. As a best practice, Git works to avoid unnecessary disk space and network traffic.

We may consider providing a similar service in later stages of the project.

## Other questions?

You can first search for existing issues in Issues. If you can't find your question, feel free to post a `[Question]`! Every question you ask is a contribution to the community, and every concern drives the project forward.
