package db

const CreateIndexQuery = `
CREATE INDEX user_id_index IF NOT EXISTS FOR (u:User) ON (u.id);
CREATE INDEX post_id_index IF NOT EXISTS FOR (p:Post) ON (p.id);
CREATE INDEX post_created_at_index IF NOT EXISTS FOR (p:Post) ON (p.createdAt);
CREATE INDEX post_likes_index IF NOT EXISTS FOR (p:Post) ON (p.likes);
CREATE INDEX post_created_at_likes_index IF NOT EXISTS FOR (p:Post) ON (p.createdAt, p.likes);
CREATE INDEX post_tags_index IF NOT EXISTS FOR (p:Post) ON (p.tags);
`

const CreateUserQuery = `
CREATE (u:User {id: $userID})
RETURN u
`

const DeleteUserQuery = `
MATCH (u:User {id: $userID})
DETACH DELETE u
`

const CreatePostQuery = `
MATCH (u:User {id: $userID})
CREATE (u)-[:CREATED]->(p:Post {id: $postID, createdAt: $createdAt, tags: $tags, likes: 0, views: 0})
RETURN p
`

const UpdatePostQuery = `
MATCH (p:Post {id: $postID})
SET p.tags = $tags
RETURN p
`

const DeletePostQuery = `
MATCH (p:Post {id: $postID})
DETACH DELETE p
`

const AddFriendQuery = `
MATCH (u1:User {id: $userID}), (u2:User {id: $friendID})
MERGE (u1)-[:FRIEND]->(u2)
MERGE (u2)-[:FRIEND]->(u1)
RETURN u1, u2
`
const RemoveFriendQuery = `
MATCH (u1:User {id: $userID})-[r:FRIEND]-(u2:User {id: $friendID})
DELETE r
`

const FollowUserQuery = `
MATCH (u1:User {id: $userID}), (u2:User {id: $followedID})
MERGE (u1)-[:FOLLOWS]->(u2)
RETURN u1, u2
`

const UnfollowUserQuery = `
MATCH (u1:User {id: $userID})-[r:FOLLOWS]->(u2:User {id: $followedID})
DELETE r
`
const UserLikedPostQuery = `
MATCH (u:User {id: $userID}), (p:Post {id: $postID})
MERGE (u)-[:LIKED]->(p)
ON CREATE SET p.likes = p.likes + 1
RETURN p
`

const UserUnlikedPostQuery = `
MATCH (u:User {id: $userID})-[r:LIKED]->(p:Post {id: $postID})
DELETE r
WITH p, CASE WHEN r IS NOT NULL THEN 1 ELSE 0 END AS wasLiked
SET p.likes = p.likes - wasLiked
RETURN p
`

const UserViewedPostQuery = `
MATCH (u:User {id: $userID}), (p:Post {id: $postID})
MERGE (u)-[:VIEWED]->(p)
ON CREATE SET p.views = p.views + 1
RETURN p
`

const GenerateRecommendationQuery = `
CALL {
    MATCH (user:User {id: $userID})-[:FRIEND]->(friend:User)-[:CREATED]->(post:Post)
    WHERE post.createdAt < $cursor AND friend.id <> user.id
    RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'friends' AS source
    ORDER BY post.createdAt DESC
    LIMIT $limit

    UNION

    MATCH (user:User {id: $userID})-[:FOLLOWS]->(followed:User)-[:CREATED]->(post:Post)
    WHERE post.createdAt < $cursor AND followed.id <> user.id
    RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'followers' AS source
    ORDER BY post.createdAt DESC
    LIMIT $limit

    UNION

    MATCH (user:User {id: $userID})-[:LIKED|VIEWED]->(likedPost:Post)<-[:CREATED]-(author:User)
    WITH user, collect(distinct likedPost.tags) AS userTags
    MATCH (post:Post)
    WHERE any(tag IN post.tags WHERE tag IN userTags) AND post.createdAt < $cursor
    RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'tags' AS source
    ORDER BY post.createdAt DESC
    LIMIT $limit

    UNION

    MATCH (user:User)-[:CREATED]->(post:Post)
    WHERE post.likes >= $minLikes AND post.createdAt < $cursor AND user.id <> $userID
    RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'popular' AS source
    ORDER BY post.likes DESC, post.createdAt DESC
    LIMIT $limit

    UNION

    MATCH (user:User)-[:CREATED]->(post:Post)
    WHERE post.createdAt < $cursor AND user.id <> $userID
    RETURN post.id AS postID, post.likes AS likes, post.createdAt AS createdAt, 'recent' AS source
    ORDER BY post.createdAt DESC
    LIMIT $limit
}

WITH postID, MAX(likes) AS likes, MAX(createdAt) AS createdAt, COLLECT(source)[0] AS source
RETURN postID, likes, createdAt, source
ORDER BY createdAt DESC, likes DESC
LIMIT $limit`
