# Permissions Bot
This example shows how to implement the permissions bot example from the [Discord documentation for subcommands and subcommand groups](https://discord.com/developers/docs/interactions/application-commands#subcommands-and-subcommand-groups). It implements the basic subcommand structure from that example:
```
command
|__ subcommand-group
    |__ subcommand
    |__ subcommand
|__ subcommand-group
    |__ subcommand
    |__ subcommand
```
The "Get Permissions for Role/User" commands work, but the "Edit Permissions for Role/User" just prints out a message. This is because the main purpose of this example is to show off the subcommand structure from the Discord docs, and to actually edit permissions would require some extra command options and the bot to have uncomfortably high-level permissions.

In addition, to actually provide a decent UX to edit permissions requires more complex features such as autocomplete for the permission name (as there are more than 25 permissions, more than the discord limit for a choices based input). In the future, we may implement a PermissionsBot V2 example which has:
- Autocomplete for the permission names when editing permissions
- Permission checks to ensure the user doing the editing has appropriate permissions