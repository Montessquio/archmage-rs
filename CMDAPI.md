
# Archmage

## Archmage Management Commands

Here's an overview of who's allowed to use which commands:

Admin commands which start with `!config` require the user to have the "Server Administrator" permission.

Game management commands which start with `!campaign` require the user to have been authorized for that
campaign by a server administrator.

Player commands which are `!inv !pot !quests !char` can be used by anyone to *read* information, but can only be used to *change* information by authorized players added by DMs (and the DMs themselves).

In this guide, `<Parameter>` means a required variable parameter
and `[Parameter]` means an optional variable parameter.

### Global Server Configuration

The default command prefix is the explamation point `!`.
It may be changed with the following command if you are a Server Administrator:

```text
!config prefix <Desired Prefix>
```

Leaving the Desired Prefix blank will set it back to the default.

## Campaign Configuration

### Creating Campaigns and Managing DMs

In order to know which channel is bound to which campaigns,
a server Administrator
must make a user a DM of a campaign. There is no command
to create a campaign - this command will simply create one
if it does not exist.

```text
!config dm <@User> <Campaign-Name>
!config remove [@User] <Campaign-Name>
```

> Be careful when using the *remove* subcommand - if no user
is specified, it will delete the entire campaign, with
no way to recover it, *instead* of simply removing a DM from it.
However, Removing the last DM from a campaign will not delete it!

### Get Campaign Info

To see the information Archmage keeps about a specific campaign, a server Administrator
or registered DM may use the following command:

```text
!campaign [Campaign-Name] info
```

Archmage will then send all the info it knows about the given campaign in response.

### Managing Player Characters

DMs may add or remove users to characters in their campaign.
Removing a character deletes their inventory from the database,
so have caution. However, items they contributed to the pot
and quests they added will not be affected.

If you specify a discord user in the remove command, it will remove that player
from the character without deleting the character. However, if you do not specify
any users, it will delete the character itself! Have caution.

```text
!campaign [Campaign-Name] add <@Player> <Character-ID>
!campaign [Campaign-Name] remove [@Player] <Character-ID>
```

The character-id is a short name identifying the character,
ideally able to be quickly and easily typed.
For example, if the character's full name is "Calmasis M. Bluthersworth",
an appropriate Character ID would be "Calmasis" or simply, "Cal".

You cannot have two characters share the same ID in a given campaign.

### Selecting channels

When a DM or Player runs a command, Archmage needs to know what campaign that command
corresponds to. They can either specify which campaign they want *every time they run a command*,
or a DM can assign their campaign to specific channels, and Archmage will just assume
that every time an ambiguous command is run there, the user means a certain campaign.

```text
!campaign <Campaign-Name> set <#channel>
!campaign <Campaign-Name> remove <#channel>
```

Using `remove` instead of set takes the campaign off of that channel.
Only one campaign may be assigned to a channel at a time.

## Enabling and Disabling Features

By default, Inventory management, character management, and quest
management are all enabled, and any user with an associated character
or DM status may use those commands. The DM may disable them with
the following commands:

```text
!campaign [Campaign-Name] disable inventory
!campaign [Campaign-Name] disable quests
!campaign [Campaign-Name] disable characters
```

If you wish to reenable these, simply replace `disable` with `enable`.

## Accessing Campaign Info

By default, player management is in
*response mode*, where players must query
their information using the following commands,
and the bot responds with the requested
data.

This assumes the command is run in an associated channel
(see above). If not, the campaign name must be included
immediately after the command (`!inv <Campaign ID> <Character ID>`).

Display your inventory:

```text
!inv <Character ID>
```

Display the party pot:

```text
!pot
!inv pot
```

Display currently active quests:

```text
!quest list
```

Display character information post
(or optionally a specific tag of the post):

```text
!character <Character ID> [Tag]
!char <Character ID> [Tag]
```

## Editing Campaign Info

Once channels are enabled for a specific campaign,
players may use Inventory management commands there
to edit their inventories.

DMs may specify any character regardless of who
they are assigned to. Players must specify a character
they are assigned to.

### Adding Items

To add a Number of Item(s) to a player's
personal inventory,

```text
!inv <Character> add [Number] <Item>
```

To add a Number of Item(s) to a party's
shared inventory,

```text
!pot add [Number] <Item>
```

If no Number is specified, it defaults to 1.
If the item is not yet in the player's inventory,
the item will be created.

### Removing Items

To decrease the quantity of a certain item from a player character's
inventory,

```text
!inv <Character> remove [Number] <Item>
```

To do the same to a party's
shared inventory,

```text
!pot remove [Number] <Item>
```

If no Number is specified, it removes the whole stack.
If the number of items reaches exactly zero, then the item is removed from the list.
The number of items may not become negative.

### Setting Items

To set exactly the number of items a player character has without
adding or subtracting,

```text
!inv <Character> set [Number] <Item>
```

To do the same to a party's
shared inventory,

```text
!pot set [Number] <Item>
```

If no Number is specified, it will return an error.

### Working With Multiple Items at a Time

When adding, removing, or setting items to an inventory or pot, you can
also specify many tags in one command, separated by
semicolons:

```text
!inv <Character ID> add [Amount] <Item>; [Amount] <Item> etc...
```

If you want to use a semicolon in your values, surround
it with double quotes:

```text
!inv <CharacterID> add "this is; one item"
```

For Example, assuming an empty inventory for a
character ID "Calmasis" to start with:

```text
!inv add Calmasis 3000 Gold
!inv add Calmasis 1 "Potion of Strength"
```

```text
!inv Calmasis
```

> **Calmasis' Inventory**
>
> 3000 Gold
>
> 1 Potion of Strength

This applies to !pot commands as well.

## Character Post Editing

The DM or associated players may add or remove
information to or from their character with commands:

```text
!char <Character ID> info add <tag> <...value>
!char <Character ID> info remove <tag>
```

An Empty Tag will not be removed!

You can also specify many tags in one command,
separated by semicolons:

```text
!char <Character ID> info add <tag> <...value>; <tag> <...value> etc...
```

If you want to use a semicolon in your values, surround
it with double quotes:

```text
!char <CharacterID> info add Example "this is; one value"
```

Both tags and values may have spaces in them, but only
if they are surrounded in double quotes.

If an Image URL is the only content in a value,
it will be embedded in any resulting queries as an image.

For example, the following commands will produce the
corresponding outputs for an imaginary character
with ID "Calmasis"

The DM Runs:

```text
!campaign NTETE add @Estarbon#0001 Calmasis
```

And then the DM or the Player runs:

```text
!char Calmasis info add Name "Calmasis M. Bluthersworth"; Alignment LN; Race "Ugol Kenku"
!char Calmasis info add "Ability Scores" "STR 14, DEX 20, CON 16, INT 27, WIS 12, CHA 10"
!char Calmasis info add Languages "Rokugani, Old Kalessian, Arakilish, Celestial, Infernal, Abyssal, Kenku, Auran, Ignan"
```

```text
!char Calmasis Name
```

> **Name**: Calmasis M. Bluthersworth

```text
!char Calmasis
```

> **Character information for Calmasis**
>
> **Name**: Calmasis M. Bluthersworth
>
> **Alignment**: LN
>
> **Race**: Ugol Kenku
>
> **Ability Scores**
>
> STR 14, DEX 20, CON 16, INT 27, WIS 12, CHA 10
>
> **Languages**
>
> Rokugani, Old Kalessian, Arakilish, Celestial, Infernal, Abyssal, Kenku, Auran, Ignan

## XP and Levels

Because XP and Levelling is often an error-prone and easily forgotten task,
Archmage seeks to make it easy for you by managing it for you!

First, the DM must set XP thresholds. For example, the following
tells Archmage that in this campaign, it takes 1,000 XP to
reach level 1:

```text
!campaign <Campaign-Name> level 1 1000
```

If you try to set a level with an XP total lower
than a previous level, Archmage will return an error.
To delete a level from the table, use the `remove` subcommand.

```text
!campaign <Campaign-Name> level remove 1
```

To view the XP Table:

```text
!campaign <Campaign-Name> show
```

Once these are set, the DM and their players can
use the `!xp` command to track and modify their XP.

By default, the XP thresholds are mapped to the D&D 3.5e SRD's standards.

To view your current XP and Level, use:

```text
!xp <Character-ID>
```

> **Character Name**
>
> **XP** **Level**
>
> \<XP Amount>  \<Level>

To edit your current XP, use:

```text
!xp <Character-ID> add <Number>
!xp <Character-ID> remove <Number>
!xp <Character-ID> set <Number>
```

As usual, players may only specify characters
they're assigned to, while the DM may specify
any character.

When a character's XP crosses a threshold,
their level is automatically adjusted in their character
post and in the !xp command.

### XP Modifiers

It is possible to automatically apply a multiplier to all added
(but not removed or set) xp.

Here is an example which adds a +2% XP modifier to a character.

```text
!xp <Character-ID> mod 1.02
```

> Set xp multiplier for \<Character> to \<Number>.

```text
!xp <Character-ID> add 100
```

> 102 XP added to \<Character>.

```text
!xp <Character-ID>
```

> \<Character> (Level 1) has 102 XP
>
> XP Multiplier: 1.02

For no multiplier, set it to 1.

## Quest Management

Quests are managed just like everything else:

## Live Updates and Tallies

The DM may enable *monitoring* in a specific channel,
where the bot will send and edit its own messages to
continuously reflect the state of the party pot,
inventories, and quests. This can be done with the
following commands.

```text
!campaign [Campaign-Name] inventory monitor [#channel]

!campaign [Campaign-Name] quests monitor [#channel]

!campaign [Campaign-Name] characters monitor [#channel]
```

This will cause the bot to actively update a list of
inventory and quests, respectively, in the specified channel.
If the channel is not specified, it assumes the
channel the message is sent in is the desired one.

Monitoring can be enabled in only one channel per type per campaign.
To disable monitoring, type `disable` instead of a channel.
