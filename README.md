# Zylo
> [!IMPORTANT]
> There is still work in progress and the current implementation does not represent the expected result and will change in the future. 

The idea behind this project was to create a social media platform that could potentially
serve thousands, even millions of users. The goal was to build a system flexible enough
to work with different technologies, and to apply the principles of microservices—small,
independent services that work together.

Inspired by platforms like Twitter (now known as X), the project aims to handle core
social media features: user accounts, personalized feeds, friend connections, and basic
interactions like likes and replies. The ultimate objective was to create a functioning,
scalable example of a modern social platform.

![image](https://github.com/user-attachments/assets/5b16d060-d400-4d67-a9ca-9c158a0cbb76)
> [!IMPORTANT]
> This is not the final architecture and you will see the architecture that should be more suitable for social media later.

## Features
- [x] Authentication (Login, Register, Account Verification)
- [x] Posts Management (CRUD + Image support)
- [x] Users Management (Get, Update, Delete + Profile/Background image support)
- [x] Social Interactions (Followers, Friends, Blocks)
- [x] User Interactions (Likes, Views, Recursive Replies)
- [x] Feed Recommendations
- [ ] Direct Messaging
- [ ] Searching for posts/hashtags/users
- [ ] Users recommendation (aka recommend friends of user's friends)
- [ ] Password reset
- [ ] Notifications

# Microservices
## User Management
Manages all aspects of user accounts—creating them, verifying passwords, providing secure access, and protecting sensitive information like emails or passwords.
All sensitive data are hashed and in case it is needed for other services, it will send message with encrypted data. For example when sending the account confirmation token to the `Notification Service`, the code will be encrypted.

![image](https://github.com/user-attachments/assets/c0b64f43-9471-42bb-bede-f2af7edb6d21)
For now, each microservice makes a copy of newly created user data, and whenever this
data changes, they must update it as well.

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

# Current Challenges
A key challenge in the current design is the complexity and inefficiency of retrieving all
the data needed to display a user’s information. For instance, loading a single user’s
profile may involve multiple steps:
1. First, the system must fetch the user’s basic details (name, bio, etc.) from the
User Management service.
2. Then, it must retrieve that user’s social connections (friends, followers) from the
Social Graph service.
3. After that, it needs to fetch the user’s posts from the Media Service.
4. Finally, for each post, the system must call the User Interaction service to gather
likes, views, and replies.
![image](https://github.com/user-attachments/assets/cf7b01d8-0683-4cbe-9e5d-0cdbb487dc07)

This results in a large number of calls. For example, if a user has created 100 posts,
the API Gateway might need to make 100 separate requests just for interaction data.
This leads to slower responses and increased system load.
![image](https://github.com/user-attachments/assets/acfc644b-cb0d-4586-a949-7f84f30cfa09)

The same issue appears in other scenarios, such as generating a user’s feed. The
Feed Service might return a list of recommended posts, and then the system must
individually request post details and interactions for each recommendation. This
“looping” pattern of repeated calls creates significant delays and unnecessary complexity.
![image](https://github.com/user-attachments/assets/93831279-4cbd-46c4-bcac-a41693bcdd10)

# Upcoming changes
![image](https://github.com/user-attachments/assets/bbef1116-a4e0-49e7-94ad-da6bbdfd5bb0)
- **Single Point of Data Assembly**: Instead of the client or gateway calling multiple
services individually, the Aggregation Service retrieves everything at once.
- **Batch Requests**: All necessary data (user info, posts, social connections,
interactions) can be requested in a single round trip, significantly reducing the
total number of calls.
- **Improved Performance with Faster Protocols**: Internally, we can use high-
speed protocols like gRPC, which move data quickly and reliably compared to
traditional REST calls.
- **Simplicity for Other Services**: Each microservice focuses solely on its core
responsibilities without needing to store extra user details or frequently call other
services. This keeps the entire system cleaner and more maintainable.

Now If the client wants to get current profile the sequence of calls would look like this
![image](https://github.com/user-attachments/assets/62f6d03c-f8ae-4e36-81db-0ce11b094a21)
Previously, retrieving a user’s profile might have required the gateway to call multiple
services—User Management, Social Graph, Media Service, and User Interaction—
resulting in numerous requests and slow response times. Now, the Aggregation Service
handles all these internal calls at once. The client sends one request to the gateway,
the gateway calls the Aggregation Service, and then the Aggregation Service returns
fully combined user data in a single response.

And the same goes for personolized feed
![image](https://github.com/user-attachments/assets/d01e4423-44ba-47dd-8f55-e6bdcec88f8a)
Generating a user’s personalized feed now involves just one main request. The
Aggregation Service fetches recommended posts, gathers the post details, and then
retrieves user interaction data (likes, views, replies) in one go. Instead of managing
dozens of separate requests, the client sees just one response containing a rich, fully
assembled feed.
**Advantages**:
- **Less Coupling**: Services no longer depend on each other for data, reducing
complexity.
- **No Extra Data Storage**: Microservices don’t need to store unrelated user details
just to answer certain requests.
- **Simplified Code**: Removing unnecessary integrations means fewer lines of code
and simpler logic.
- **Fewer Network Calls**: Batch requests and a single aggregator reduce network
load.
- **Flexible and Fast**: All the “merging” logic sits in one place, making it easier to
optimize and even experiment with different technologies for maximum speed.
