package storage

const CreateIndexQuery = `
		CREATE INDEX node_range_index_id IF NOT EXISTS
		FOR (u:User) ON (u.id)`

const GetUserWithRelationshipQuery = `
		MATCH (u:User {id: $userID})
				OPTIONAL MATCH (follower:User)-[:FOLLOWS]->(u)
				OPTIONAL MATCH (u)-[:FOLLOWS]->(followed:User)
				OPTIONAL MATCH (u)-[:BLOCKS]->(blocked:User)
				OPTIONAL MATCH (u)-[:FRIEND]->(friend:User)
				OPTIONAL MATCH (u)-[:FRIEND_REQUEST]->(pendingRequestReceiver:User)
				OPTIONAL MATCH (pendingRequestSender:User)-[:FRIEND_REQUEST]->(u)
				RETURN u, 
					   COLLECT(DISTINCT follower) AS followers,
					   COLLECT(DISTINCT followed) AS followedPeople,
					   COLLECT(DISTINCT blocked) AS blockedPeople,
					   COLLECT(DISTINCT friend) AS friends,
					   COLLECT(DISTINCT pendingRequestReceiver) AS sentFriendRequests,
					   COLLECT(DISTINCT pendingRequestSender) AS receivedFriendRequests`

const GetFollowersQuery = `
		MATCH (u1:User {id: $userID})<-[:FOLLOWS]-(u2:User) RETURN u2`

const GetFollowedPeopleQuery = `
		MATCH (u1:User {id: $userID})-[:FOLLOWS]->(u2:User) RETURN u2`

const GetBlockedPeopleQuery = `
		MATCH (u1:User {id: $userID})-[:BLOCKED]->(u2:User) RETURN u2`

const GetFriendsQuery = `
		MATCH (u1:User {id: $userID})-[:FRIEND]-(u2:User) 
		RETURN DISTINCT (u2)`

const GetPendingFriendRequestsQuery = `
		MATCH (u1:User {id: $userID})<-[:FRIEND_REQUEST]-(u2:User) 
		RETURN u2`

const CreateUserQuery = `
		CREATE (u:User {id: $id, username: $username,name: $name, created_at: $created_at})`

const UpdateUserQuery = `
		MATCH (u:User {id: $id})
		SET u.name = $name,
			u.bio = $bio,
			u.location = $location`

const DeleteUserByIDQuery = `
		MATCH (u:User {id: $userID})
		DETACH DELETE u`

const FollowUserQuery = `
		MATCH (u1:User {id: $followerID}), (u2:User {id: $followedID})
		WHERE NOT EXISTS ((u1)-[:FOLLOWS]->(u2))
		CREATE (u1)-[:FOLLOWS]->(u2)`

const UnfollowUserQuery = `
		MATCH (follower:User {id: $followerID})-[r:FOLLOWS]->(followed:User {id: $followedID}) 
		DELETE r`

const SendFriendRequestQuery = `
		MATCH (u1:User {id: $userID}), (u2:User {id: $receiverID})
		WHERE NOT EXISTS ((u1)-[:FRIENDS]-(u2)) AND NOT EXISTS ((u1)-[:FRIEND_REQUEST]-(u2))
		CREATE (u1)-[:FRIEND_REQUEST]->(u2)
		RETURN u1, u2`

const AcceptFriendRequestQuery = `
		MATCH (sender:User {id: $userID})<-[r:FRIEND_REQUEST]-(receiver:User {id: $receiverID})
        DELETE r
        CREATE (sender)-[:FRIEND]->(receiver), (receiver)-[:FRIEND]->(sender)`

const DeclineFriendRequestQuery = `
		MATCH (sender:User {id: $userID})<-[r:FRIEND_REQUEST]-(receiver:User {id: $receiverID}) 
		DELETE r`

const RemoveFriendQuery = `
		MATCH (u1:User {id: $userID})-[r:FRIEND]-(u2:User {id: $friendID}) 
		DELETE r`

const BlockUserQuery = `
		MATCH (u1:User {id: $blockerID}), (u2:User {id: $blockedUserID})
		OPTIONAL MATCH (u1)-[r]-(u2)
		DELETE r
		CREATE (u1)-[:BLOCKED]->(u2)`

const UnblockUserQuery = `
		MATCH (u1:User {id: $blockerID})-[r:BLOCKED]->(u2:User {id: $blockedUserID}) DELETE r`
