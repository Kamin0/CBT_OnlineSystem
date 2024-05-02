# Général
  
# Endpoints
  


------------------------------------------------------------------------------------------
 
### Test Server Avaibility
<details>
<summary><code>GET</code> <code><b>/hello</b></code> ➡️ <code>{ALL : Ping server}</code></summary>

#### Parameters
- Auth required : None

#### Success Response
Code : `200 OK`
Content example
```
Hello, World !
```

#### Error Responses
- None

</details>


------------------------------------------------------------------------------------------
 
### Authentification
<details>
<summary><code>POST</code> <code><b>/register</b></code>  ➡️  <code>{CLIENT & SERVER : Register Client or Server}</code></summary>

#### Data constraints
```json
{
  "username": "user",
  "email": "test@etu.uqac.ca",
  "password": "test0",
  "role_name": "client" OR "server"
}
```
- Auth required : None

#### Success Response
Code : `200 OK`
```json
"User registered successfully"
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Invalid role name provided`                                        |
> | `400`         | `application/json`                | `Error inserting user into database`                                |

</details>

<details>
<summary><code>POST</code> <code><b>/login</b></code>  ➡️ <code>{CLIENT & SERVER : Login}</code></summary>

#### Data constraints
```json
{
  "username": "{user}",
  "password": "{password}"
}
```

- Auth required : None

#### Success Response
Code : `200 OK`
```json
{token_client} OR {token_server}
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Invalid username or password`                                      |
> | `400`         | `application/json`                | `Invalid role id`                                                   |
</details>

------------------------------------------------------------------------------------------
 
### Sessions

<details>
<summary><code>GET</code> <code><b>/session/{username}</b></code>  ➡️ <code>{CLIENT: Request session id for client}</code></summary>

#### Parameters

> | name              |  type     | data type      | description                         |
> |-------------------|-----------|----------------|-------------------------------------|
> | username          |  required | string         |                                     |

- Auth required : `token_client`

#### Success Response
Code : `200 OK`
Content example
```json
{session_id}
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `No session available`                                              |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Invalid rank id`                                                   |
> | `400`         | `application/json`                | `Session not found`                                                 |
> | `400`         | `application/json`                | `Failed to connect to Redis`                                        |
> | `400`         | `application/json`                | `Unauthorized`                                                      |

</details>

<details>
<summary><code>POST</code> <code><b>/session</b></code>  ➡️ <code>{SERVER: Register session}</code></summary>

#### Data constraints
```json
{
    "server_address" : "{{server_adress}}",
    "players" : []
}
```
- Auth required : `token_server`

#### Success Response
Code : `200 OK`
Content example
```json
Session registered successfully
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error inserting user into database`                                |
> | `400`         | `application/json`                | `Failed to connect to Redis`                                        |
> | `400`         | `application/json`                | `Unauthorized`                                                      |

</details>


<details>
<summary><code>DELETE</code> <code><b>/session</b></code>  ➡️ <code>{SERVER: Delete current session}</code></summary>

- Auth required : `token_server`

#### Success Response
Code : `200 OK`
Content example
```json
Session successfully deleted
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error updating database`                                           |
> | `400`         | `application/json`                | `Failed to connect to Redis`                                        |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>



<details>
<summary><code>POST</code> <code><b>/connect</b></code>  ➡️ <code>{CLIENT: Connect to a specific session}</code></summary>

- Auth required : `token_client`

#### Data constraints
```json
{
    "session_id" : "{session_id}",
    "username" : "{username}"
}
```

#### Success Response
Code : `200 OK`
Content example
```json
Player connected to session successfully
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `No session available`                                              |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Session not found`                                                 |
> | `400`         | `application/json`                | `Failed to connect to Redis`                                        |
> | `400`         | `application/json`                | `Error updating session`                                            |
> | `400`         | `application/json`                | `Unauthorized`                                                      |

</details>

------------------------------------------------------------------------------------------
 
### Achievements
<details>
<summary><code>POST</code> <code><b>/achievement</b></code>  ➡️ <code>{SERVER: Validate Achievement}</code></summary>

- Auth required : `token_server`

#### Data constraints
```json
{
    "username" : "{username}",
    "achievement_id" :  "{achievement_id}"
}
```

#### Success Response
Code : `200 OK`
Content example
```json
Achievement validated successfully
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Session not found`                                                 |
> | `400`         | `application/json`                | `Error inserting user achievement into database`                    |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>



<details>
<summary><code>GET</code> <code><b>/achievement/{achievement_id}</b></code>  ➡️ <code>{CLIENT: Get achievement by id}</code></summary>

#### Parameters

> | name              |  type     | data type      | description                         |
> |-------------------|-----------|----------------|-------------------------------------|
> | achievement_id          |  required | string         |    achievement id             |

- Auth required : `token_client`

#### Success Response
Code : `200 OK`
Content example
```json
{
    "id" :  "{achievement_id}",
    "name" : "{achievement_name}",
    "description" : "{description}",
    "image_url" : {}
}
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Achievement not found`                                             |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>


<details>
<summary><code>GET</code> <code><b>/achievements</b></code>  ➡️ <code>{SERVER & CLIENT: Get all achievements}</code></summary>

- Auth required : `token_client` OR `token_server`

#### Success Response
Code : `200 OK`
Content example
```json
[
    {
        "id" :  "{achievement_id}",
        "name" : "{achievement_name}",
        "description" : "{description}",
        "image_url" : {}
    }, 
    "..."
}
```

#### Error Responses

> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error loading achievements`                                        |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>GET</code> <code><b>/user_achievements/{username}</b></code>  ➡️ <code>{SERVER & CLIENT: Get all achievements of a specific player}</code></summary>

#### Parameters
> | name              |  type     | data type      | description                         |
> |-------------------|-----------|----------------|-------------------------------------|
> | username          |  required | string         |    player username                  |


- Auth required : `token_client` OR `token_server`
#### Success Response
Code : `200 OK`
Content example
```json
[
    {
        "id" :  "{achievement_id}",
        "name" : "{achievement_name}",
        "description" : "{description}",
        "image_url" : {}
    }, 
    "..."
}
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error loading achievements`                                        |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>



------------------------------------------------------------------------------------------
### Rank & Stats




<details>
<summary><code>GET</code> <code><b>/ranks</b></code>  ➡️ <code>{SERVER & CLIENT: Get all ranks}</code></summary>


- Auth required : `token_client` OR `token_server`
#### Success Response
Code : `200 OK`
Content example
```json
[
    {
        "id" :  "{rank_id}",
        "name" : "{rank_name}",
        "image_url" : {}
    }, 
    "..."
}
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error loading ranks`                                               |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>GET</code> <code><b>/rank/{username}</b></code>  ➡️ <code>{SERVER & CLIENT: Get rank of a specific user}</code></summary>

#### Parameters
> | name              |  type     | data type      | description                         |
> |-------------------|-----------|----------------|-------------------------------------|
> | username          |  required | string         |    player username                  |



- Auth required : `token_client` OR `token_server`
#### Success Response
Code : `200 OK`
Content example
```json
{
    "rank_name"
}, 
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Failed to retrieve rank name`                                      |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>


<details>
<summary><code>GET</code> <code><b>/ranks</b></code>  ➡️ <code>{SERVER & CLIENT: Get all ranks}</code></summary>


- Auth required : `token_client` OR `token_server`
#### Success Response
Code : `200 OK`
Content example
```json
[
    {
        "id" :  "{rank_id}",
        "name" : "{rank_name}",
        "image_url" : {}
    }, 
    "..."
}
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error loading ranks`                                               |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>PUT</code> <code><b>/rank</b></code>  ➡️ <code>{SERVER: Update rank of a specific user}</code></summary>

#### Data constraints
```json
{
    "username" : "{username}",
    "new_rank_id" : "{rank id}"
}
```
- Auth required : `token_server`



#### Success Response
Code : `200 OK`
Content example
```json
Rank updated successfully
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Failed to retrieve rank name`                                      |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>GET</code> <code><b>/kda/{username}</b></code>  ➡️ <code>{SERVER & CLIENT: Get kda of a specific user}</code></summary>

#### Parameters
> | name              |  type     | data type      | description                         |
> |-------------------|-----------|----------------|-------------------------------------|
> | username          |  required | string         |    player username                  |



- Auth required : `token_client` OR `token_server`
#### Success Response
Code : `200 OK`
Content example
```json
{kda}, 
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>PUT</code> <code><b>/kda</b></code>  ➡️ <code>{SERVER: Update kda of a specific user}</code></summary>

#### Data constraints
```json
{
    "username" : "{username}",
    "new_kda" : {new_kda}
}
```
- Auth required : `token_server`



#### Success Response
Code : `200 OK`
Content example
```json
KDA updated successfully
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error updating user kda         `                                  |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>GET</code> <code><b>/nb_games/{username}</b></code>  ➡️ <code>{SERVER & CLIENT: Get number of game played by a specific user}</code></summary>

#### Parameters
> | name              |  type     | data type      | description                         |
> |-------------------|-----------|----------------|-------------------------------------|
> | username          |  required | string         |    player username                  |



- Auth required : `token_client` OR `token_server`
#### Success Response
Code : `200 OK`
Content example
```json
{nb_games_played}, 
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>PUT</code> <code><b>/nb_games/{username}</b></code>  ➡️ <code>{SERVER: Increment number of game played by a specific user}</code></summary>

- Auth required : `token_server`


#### Success Response
Code : `200 OK`
Content example
```json
Games played updated successfully
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error updating user games played`                                  |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>



------------------------------------------------------------------------------------------
### Friends


<details>
<summary><code>POST</code> <code><b>/send_friend_request</b></code>  ➡️ <code>{CLIENT: Send friend request}</code></summary>

#### Data constraints
```json
{
    "username": "{player sending request username}",
    "friend_username": "{asked friend username}"
}
```
- Auth required : `token_client`



#### Success Response
Code : `200 OK`
Content example
```json
Friend request sent successfully
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error inserting friend request into database`                      |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Invalid friend username`                                           |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>POST</code> <code><b>/accept_friend_request</b></code>  ➡️ <code>{CLIENT: Accept a specific friend request}</code></summary>

#### Data constraints
```json
{
    "username": "{player accepting request username}",
    "friend_username": "{new friend username}"
}
```
- Auth required : `token_client`



#### Success Response
Code : `200 OK`
Content example
```json
Friend request accepted successfully
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error inserting friend into database`                              |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Invalid friend username`                                           |
> | `400`         | `application/json`                | `Error removing friend request`                                     |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>DELETE</code> <code><b>/friend</b></code>  ➡️ <code>{CLIENT: Delete a specific friend}</code></summary>

#### Data constraints
```json
{
    "username": "{initiating player}",
    "friend_username": "{deleted friend}"
}
```
- Auth required : `token_client`



#### Success Response
Code : `200 OK`
Content example
```json
Friend removed successfully
```

#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error removing friend`                              |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Invalid friend username`                                           |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>GET</code> <code><b>/friends/{username}</b></code>  ➡️ <code>{CLIENT: Get all all friends of a specific player}</code></summary>

#### Parameters
> | name              |  type     | data type      | description                         |
> |-------------------|-----------|----------------|-------------------------------------|
> | username          |  required | string         |    player username                  |


- Auth required : `token_client`



#### Success Response
Code : `200 OK`
Content example
```json
[
    {
        "username" :  "{}",
        "kda" : "{}",
        "nb_games" : {},
        "rank" : {}
    }, 
    "..."
}
```


#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error loading friends`                              |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>

<details>
<summary><code>GET</code> <code><b>/get_friend_requests/{username}</b></code>  ➡️ <code>{CLIENT: Get all all pending friends requests of a specific player}</code></summary>

#### Parameters
> | name              |  type     | data type      | description                         |
> |-------------------|-----------|----------------|-------------------------------------|
> | username          |  required | string         |    player username                  |


- Auth required : `token_client`



#### Success Response
Code : `200 OK`
Content example
```json
[
    {
        "username" :  "{}",
        "kda" : "{}",
        "nb_games" : {},
        "rank" : {}
    }, 
    "..."
}
```


#### Error Responses
> | http code     | content-type                      | response                                                            |
> |---------------|-----------------------------------|---------------------------------------------------------------------|
> | `400`         | `application/json`                | `Content type error`                                                |
> | `400`         | `application/json`                | `Error loading friends requests`                              |
> | `400`         | `application/json`                | `Invalid username`                                                  |
> | `400`         | `application/json`                | `Unauthorized`                                                      |
</details>


