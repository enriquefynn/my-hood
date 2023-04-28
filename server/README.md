## Examples of queries:


Create a user:

```
mutation {
  createUser(input: {name: "F43off13", 
    address: "Rua a asd",
    email: "foo@ba33r.com",
    birthday:"2023-04-28T18:37:58.523027",
    signedAt: "2023-04-28T18:37:58.523027",
    identities: "cpf:232323232, rg:213123123"}) {
    id, name
  }
}
```


Query a user:

```
# Fetch user id
{
  user(input: {id:"dda0fe50-76a5-4fba-8c8d-9c96064e0959"})
  {
    id, name, email, updatedAt
  }
}
```