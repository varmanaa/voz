# Voz
Voz is a Discord app that helps manage voice channels within your Discord server. Once set up, members will have a variety of options to help curate their voice channel experience.

With Voz, members may:
- allow, deny, and remove permission for a member to join a voice channel,
- modify the bitrate,
- claim or transfer ownership of a voice channel,
- delete the voice channel,
- modify the name,
- modify a channel to be temporary (the channel is deleted when empty) or permanent (the channel remains unless otherwise set),
- lock, unlock, or completely hide a voice channel from everyone,
- modify the slow mode (duration),
- modify the user limit,
- view a voice channel's current settings,
- and modify the voice region

## Commands
Voz only has two commands - `/join` and `/voice` - that are used to manage join and voice channels, respectively.
> [!NOTE]
> By default, `/join` is initially locked to users with the `ADMINISTRATOR` permission. This may be changed within a server's settings; however, do so at your own risk.

### `/join` channel(s)
Join channels are voice channels through which users will create their own voice channels. Through Voz, server administrators (or other allowed users) may configure the following properties of any join channel:
<table>
  <tr>
    <th>Property</th>
    <th>Description</th>
  </tr>
  <tr>
    <td>Access role</td>
    <td>The role to gate the channel behind</td>
  </tr>
  <tr>
    <td>Category</td>
    <td>The category channel to create voice channels under</td>
  </tr>
  <tr>
    <td>Name</td>
    <td>The name of the channel</td>
  </tr>
  <tr>
    <td>Permanence</td>
    <td>Whether (or not) created voice channels should be deleted when empty</td>
  </tr>
  <tr>
    <td>Privacy</td>
    <td>
      The visibility level of the channel
      <ul>
        <li><strong>Invisible</strong> - the channel is completely hidden</li>
        <li><strong>Locked</strong> - the channel is visible, but locked</li>
        <li><strong>Unlocked</strong> - the channel is visible and not locked</li>
      </ul>
    </td>
  </tr>
</table>

The following subcommands are available to configure a join channel:
<table>
  <tr>
    <th>Subcommand</th>
    <th>Description</th>
  </tr>
  <tr>
    <td><code>access-role</code></td>
    <td>Modify the access role for a join channel</td>
  </tr>
  <tr>
    <td><code>category</code></td>
    <td>Modify the category (to create voice channels under)</td>
  </tr>
  <tr>
    <td><code>create</code></td>
    <td>
      <p>Create a join channel</p>
      <strong>Options</strong>
      <ul>
        <li>name - the channel name</li>
        <li>access-role - the role to let access the join channel</li>
        <li>category - the category to create voice channels under</li>
        <li>permanence - should voice channels remain when empty?</li>
        <li>privacy - the privacy option for created voice channels</li>
      </ul>
    </td>
  </tr>
  <tr>
    <td><code>name</code></td>
    <td>Modify the name of a join channel</td>
  </tr>
  <tr>
    <td><code>permanence</code></td>
    <td>Modify the permanence value of a join channel</td>
  </tr>
  <tr>
    <td><code>privacy</code></td>
    <td>Modify the privacy level of a join channel</td>
  </tr>
  <tr>
    <td><code>remove</code></td>
    <td>Remove a join channel</td>
  </tr>
  <tr>
    <td><code>view</code></td>
    <td>View the current settings of a join channel</td>
  </tr>
</table>

### `/voice` channel(s)
Members can create their own voice channel by connecting to an accessible join channel. If a member does not already own a voice channel, the member will be moved into their own voice channel with the same privacy level and permanence value as the originating join channel (as initial values).

> [!NOTE]
> Category channels can contain a limit of 50 channels. If the designated category for a join channel already has 50 channels, members will not be moved into their own voice channel and remain connected to the join channel.

The following subcommands are available to configure a voice channel:
<table>
  <tr>
    <th>Subcommand</th>
    <th>Description</th>
  </tr>
  <tr>
    <td><code>allow-member</code></td>
    <td>Allow a member permission to join your voice channel</td>
  </tr>
  <tr>
    <td><code>bitrate</code></td>
    <td>Modify the bitrate of your voice channel</td>
  </tr>
  <tr>
    <td><code>claim</code></td>
    <td>Claim an unowned voice channel</td>
  </tr>
  <tr>
    <td><code>delete</code></td>
    <td>Delete your voice channel</td>
  </tr>
  <tr>
    <td><code>deny-member</code></td>
    <td>Deny a member permission to join your voice channel</td>
  </tr>
  <tr>
    <td><code>name</code></td>
    <td>Modify the name of your voice channel</td>
  </tr>
  <tr>
    <td><code>permanence</code></td>
    <td>Modify the permanence value of your voice channel</td>
  </tr>
  <tr>
    <td><code>privacy</code></td>
    <td>Modify the privacy level of your voice channel</td>
  </tr>
  <tr>
    <td><code>remove-member</code></td>
    <td>Remove a member's permission to join your voice channel (and disconnect the member)</td>
  </tr>
  <tr>
    <td><code>slow-mode</code></td>
    <td>Modify the slow mode duration of your voice channel</td>
  </tr>
  <tr>
    <td><code>transfer</code></td>
    <td>Transfer ownership of your voice channel to another member</td>
  </tr>
  <tr>
    <td><code>user-limit</code></td>
    <td>Modify the user limit of your voice channel</td>
  </tr>
  <tr>
    <td><code>video-quality-mode</code></td>
    <td>Modify the video quality mode your voice channel</td>
  </tr>
  <tr>
    <td><code>view</code></td>
    <td>View the current settings of your voice channel</td>
  </tr>
  <tr>
    <td><code>voice-region</code></td>
    <td>Modify the voice region of your voice channel</td>
  </tr>
</table>