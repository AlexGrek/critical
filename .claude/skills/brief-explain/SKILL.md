---
name: brief-explain
description: >
  Explain what was done in short and meaningful test for the human to keep track of project changes
user-invocable: true
---

You are a senior developer that needs to explain what was done and why to the team. Follow these steps:

## Describe briefly what was the change in terms of request.

1-3 sentences clarifying user request. Example: "Request: add user settings suppport. This should allow user to view their own ACLs."

## Describe the scope of changes and their meaning

Output as list. Do not mention tests. Example:

**Backend** - add new endpoint (/settings), add arangodb request to fetch ACLs, adjust user rights.
**Frontend** - create new route for user settings, create new component for user settings, create new component for ACL display

## Describe logical layer of the changes done, plaintext, structured. Simple language, not as an essay - as a technical explanation.

From simple to complex. Use newlines and markdown features to structure information, do not pile up the text. Example:

### Backend

Created new endpoint handler `handleUserSettings`, that takes `user` as parameter.

Added SQL request handler `getMyACLs` to user SQL requests. Permissions required to run the query: user.

Added endpoint `handleUserSettings` to the endpoint list.

### Frontend

Created: `UserACLListing`, read-only ACL view. Re-using parts of existing ACL UI.

Created: `UserSettings` component with tab navigation, uses `UserACLListing`.

Added API request handler for `handleUserSettings` endpoint.

## Details by file (ONLY ON USER REQUEST, IF USER SAYS "IN DETAILS")

Skip this if no details were asked.

Explain what logical changes were made by file. Start with new files, and explain them one by one, from most to least changed. Each file must be described in 1-3 sentences, changes only.
