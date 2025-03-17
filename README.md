# Zylo
> [!IMPORTANT]
> There is still work in progress and the current implementation does not represent the expected result and will change in the future. 

![image](https://github.com/user-attachments/assets/bbef1116-a4e0-49e7-94ad-da6bbdfd5bb0)

The idea behind this project was to create a social media platform that could potentially
serve thousands, even millions of users. The goal was to build a system flexible enough
to work with different technologies, and to apply the principles of microservices—small,
independent services that work together.

Inspired by platforms like Twitter (now known as X), the project aims to handle core
social media features: user accounts, personalized feeds, friend connections, and basic
interactions like likes and replies. The ultimate objective was to create a functioning,
scalable example of a modern social platform.



# Microservices
## User Management
Manages all aspects of user accounts—creating them, verifying passwords, providing secure access, and protecting sensitive information like emails or passwords.
All sensitive data are hashed and in case it is needed for other services, it will send message with encrypted data. For example when sending the account confirmation token to the `Notification Service`, the code will be encrypted.

![image](https://github.com/user-attachments/assets/c0b64f43-9471-42bb-bede-f2af7edb6d21)


## Social Graph
Manages user's social interactions including `friends`, `followers` and `blocks`. 

![image](https://github.com/user-attachments/assets/afc013b5-5b7e-4095-b2a3-d071254d6eb8)

- When someone follows another user, the Feed Service can update that user’s
feed recommendations accordingly.
- The Notification Service can alert users when they get new friend requests or
when those requests are accepted.

## Media Service
Provides with CRUD Api for post management.

![image](https://github.com/user-attachments/assets/fbac3e5e-9859-4f47-b5f8-d60bf459b340)
- When a post is liked, the Feed Service can highlight that post for other followers,
and the Notification Service can alert the author or related users. Feed service
will try to look for similar posts based on author and content of the post.
- Notification service can send alerts to user that has created the post/reply about
new reply or like.

## Feed Service
Generates each user’s personalized feed. This includes blending posts from
friends, followed accounts, and recommended content based on the user’s past
behavior.
![image](https://github.com/user-attachments/assets/86497981-16cf-44c2-8e93-54c4c86a2526)
- Does not send anything

## Notification Service
> [!WARNING]
> Not fully implemented.

Alerts users about important events, such as new friend requests, replies to
their posts, or messages from the platform.
![image](https://github.com/user-attachments/assets/7dbe0d09-23e6-4dc0-ba7e-b562016b743c)
- Does not send anything
