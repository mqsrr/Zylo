// mockData.ts

import {
    User,
    Post,
    Reply,
    FileMetadata,
    UserRelationship,
    UserSummary,
    UserPost,
    PaginatedResponse,
} from "../models";

// Arrays to store users, posts, and replies
const users: User[] = [];
const posts: Post[] = [];
const replies: Reply[] = [];

// Function to generate a random UserSummary
function generateUserSummary(user: User): UserSummary {
    return {
        id: user.id,
        username: user.username,
        createdAt: user.createdAt.toISOString(),
    };
}

// Function to generate a random UserPost
function generateUserPost(user: User): UserPost {
    return {
        id: user.id,
        profileImage: user.profileImage,
        name: user.name,
        username: user.username,
        bio: user.bio,
        location: user.location,
    };
}

// Function to generate a random user
function generateUser(id: number): User {
    const userId = `user${id}`;
    const user: User = {
        id: userId,
        username: `user${id}`,
        profileImage: {
            fileName: `profile${id}.jpg`,
            contentType: "image/jpeg",
            url: `https://picsum.photos/seed/bg${id}/200/300.jpg`,
        },
        backgroundImage: {
            fileName: `background${id}.jpg`,
            contentType: "image/jpeg",
            url: `https://picsum.photos/seed/bg${id}/200/300.jpg`,
        },
        name: `User ${id}`,
        bio: `Bio of user ${id}`,
        location: `City ${id}`,
        birthDate: new Date("1990-01-01"),
        relationships: {
            followers: [],
            followedPeople: [],
            blockedPeople: [],
            friends: [],
            sentFriendRequests: [],
            receivedFriendRequests: [],
        },
        posts: {
            data: [],
            hasNextPage: false,
            next: "",
            perPage: 0,
        },
        createdAt: new Date(),
    };
    return user;
}

// Generate 10 users (including the loginUser)
for (let i = 1; i <= 10; i++) {
    const user = generateUser(i);
    users.push(user);
}

// Set the first user as the loginUser
const loginUser = users[0];

// Function to generate relationships between users
function generateUserRelationships() {
    // For each user, randomly assign followers, following, friends, etc.
    users.forEach((user) => {
        // Generate random number of followers
        const followerCount = Math.floor(Math.random() * users.length);
        const followersSet = new Set<UserSummary>();

        while (followersSet.size < followerCount) {
            const randomUser = users[Math.floor(Math.random() * users.length)];
            if (randomUser.id !== user.id) {
                followersSet.add(generateUserSummary(randomUser));
            }
        }
        user.relationships.followers = Array.from(followersSet);

        // Generate random number of people the user is following
        const followingCount = Math.floor(Math.random() * users.length);
        const followingSet = new Set<UserSummary>();

        while (followingSet.size < followingCount) {
            const randomUser = users[Math.floor(Math.random() * users.length)];
            if (randomUser.id !== user.id) {
                followingSet.add(generateUserSummary(randomUser));
            }
        }
        user.relationships.followedPeople = Array.from(followingSet);

        // Generate friends (mutual following)
        const friendsSet = new Set<UserSummary>();
        user.relationships.followers!.forEach((follower) => {
            if (
                user.relationships.followedPeople!.some(
                    (following) => following.id === follower.id
                )
            ) {
                friendsSet.add(follower);
            }
        });
        user.relationships.friends = Array.from(friendsSet);

        // Remove friends from followers and following lists
        user.relationships.followers = user.relationships.followers!.filter(
            (follower) => !friendsSet.has(follower)
        );
        user.relationships.followedPeople = user.relationships.followedPeople!.filter(
            (following) => !friendsSet.has(following)
        );

        // Generate sent friend requests
        const sentRequestsCount = Math.floor(Math.random() * (users.length - friendsSet.size));
        const sentRequestsSet = new Set<UserSummary>();

        while (sentRequestsSet.size < sentRequestsCount) {
            const randomUser = users[Math.floor(Math.random() * users.length)];
            if (
                randomUser.id !== user.id &&
                !friendsSet.has(generateUserSummary(randomUser)) &&
                !user.relationships.followedPeople!.some((u) => u.id === randomUser.id)
            ) {
                sentRequestsSet.add(generateUserSummary(randomUser));
            }
        }
        user.relationships.sentFriendRequests = Array.from(sentRequestsSet);

        // Generate received friend requests
        const receivedRequestsCount = Math.floor(Math.random() * (users.length - friendsSet.size));
        const receivedRequestsSet = new Set<UserSummary>();

        while (receivedRequestsSet.size < receivedRequestsCount) {
            const randomUser = users[Math.floor(Math.random() * users.length)];
            if (
                randomUser.id !== user.id &&
                !friendsSet.has(generateUserSummary(randomUser)) &&
                !user.relationships.followers!.some((u) => u.id === randomUser.id)
            ) {
                receivedRequestsSet.add(generateUserSummary(randomUser));
            }
        }
        user.relationships.receivedFriendRequests = Array.from(receivedRequestsSet);

        // Generate blocked users
        const blockedCount = Math.floor(Math.random() * 3); // Up to 2 blocked users
        const blockedSet = new Set<UserSummary>();

        while (blockedSet.size < blockedCount) {
            const randomUser = users[Math.floor(Math.random() * users.length)];
            if (randomUser.id !== user.id) {
                blockedSet.add(generateUserSummary(randomUser));
            }
        }
        user.relationships.blockedPeople = Array.from(blockedSet);
    });

    // Ensure mutual relationships where appropriate
    users.forEach((user) => {
        // Update other users' relationships based on sent friend requests
        user.relationships.sentFriendRequests!.forEach((sentRequest) => {
            const receiver = users.find((u) => u.id === sentRequest.id);
            if (receiver) {
                receiver.relationships.receivedFriendRequests!.push(generateUserSummary(user));
            }
        });

        // Update other users' relationships based on followers
        user.relationships.followedPeople!.forEach((following) => {
            const followedUser = users.find((u) => u.id === following.id);
            if (followedUser) {
                followedUser.relationships.followers!.push(generateUserSummary(user));
            }
        });

        // Update other users' relationships based on friends
        user.relationships.friends!.forEach((friend) => {
            const friendUser = users.find((u) => u.id === friend.id);
            if (friendUser) {
                if (!friendUser.relationships.friends!.some((u) => u.id === user.id)) {
                    friendUser.relationships.friends!.push(generateUserSummary(user));
                }
                // Remove from followers and following lists
                friendUser.relationships.followers = friendUser.relationships.followers!.filter(
                    (follower) => follower.id !== user.id
                );
                friendUser.relationships.followedPeople = friendUser.relationships.followedPeople!.filter(
                    (following) => following.id !== user.id
                );
            }
        });
    });
}

// Generate user relationships
generateUserRelationships();

// Function to generate a random reply, possibly with nested replies
function generateReply(depth: number = 0): Reply {
    const replyId = `reply${Math.floor(Math.random() * 1000000)}`;
    const randomUser = users[Math.floor(Math.random() * users.length)];

    const reply: Reply = {
        id: replyId,
        user: generateUserPost(randomUser),
        replyToId: "", // This will be set when attaching to a post or parent reply
        content: `This is a reply by ${randomUser.username}.`,
        nestedReplies: [],
        likes: Math.floor(Math.random() * 50),
        views: Math.floor(Math.random() * 500),
        createdAt: new Date(),
        userInteracted: Math.random() < 0.5,
    };

    // Optionally add nested replies (limit recursion depth)
    if (depth < 2 && Math.random() < 0.5) {
        const nestedReplyCount = Math.floor(Math.random() * 2) + 1; // 1 or 2 nested replies
        for (let i = 0; i < nestedReplyCount; i++) {
            const nestedReply = generateReply(depth + 1);
            nestedReply.replyToId = reply.id;
            reply.nestedReplies.push(nestedReply);
        }
    }

    return reply;
}

// Generate 20 posts for the feed
for (let i = 1; i <= 20; i++) {
    const postId = `post${i}`;

    // Generate a createdAt timestamp spread out over the last 20 days
    const createdAtDate = new Date();
    createdAtDate.setDate(createdAtDate.getDate() - i); // Posts from today back to 20 days ago

    const randomUser = users[Math.floor(Math.random() * users.length)];

    const post: Post = {
        id: postId,
        text: `This is sample post number ${i}.`,
        likes: Math.floor(Math.random() * 100),
        views: Math.floor(Math.random() * 1000),
        user: generateUserPost(randomUser),
        filesMetadata: [
            {
                fileName: `image${i}.jpg`,
                contentType: "image/jpeg",
                url: `https://picsum.photos/seed/bg${postId}/200/300.jpg`,
            },
            {
                fileName: `image${i + 1}.jpg`,
                contentType: "image/jpeg",
                url: `https://picsum.photos/seed/bg${postId}/200/300.jpg`,
            },
            {
                fileName: `image${i + 2}.jpg`,
                contentType: "image/jpeg",
                url: `https://picsum.photos/seed/bg${postId}/200/300.jpg`,
            },
        ],
        replies: [],
        createdAt: createdAtDate.toISOString(),
        updatedAt: createdAtDate.toISOString(),
        userInteracted: i % 2 === 0, // User interacted with every second post
    };

    // Generate random replies for the post
    const replyCount = Math.floor(Math.random() * 3) + 1; // 1 to 3 replies
    for (let j = 0; j < replyCount; j++) {
        const reply = generateReply();
        reply.replyToId = post.id;
        post.replies!.push(reply);
        replies.push(reply);
    }

    posts.push(post);

    // Add post to the user's posts
    const user = users.find((u) => u.id === randomUser.id);
    if (user) {
        user.posts!.data.push(post);
    }
}

export { users, posts, replies, loginUser };
