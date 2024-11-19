import express from "express";
import cors from "cors";
import bodyParser from "body-parser";
import { users, posts } from "./data/mockData";

const app = express();
app.use(cors({
  origin: 'http://localhost:5173',
  credentials: true,

}));
app.use(bodyParser.json());

app.use((req, res, next) => {
    const authHeader = req.headers["authorization"];
    if (authHeader || req.url.includes("auth")) {
        next();
    } else {
        res.status(401).json({ message: "Unauthorized" });
    }
});

// Authentication Endpoints
app.post("/auth/register", (req, res) => {
    // Simulate user registration
    res.status(201).json({
        success: true,
        id: "user1",
        accessToken: {
            value: "mockAccessToken",
            expirationDate: new Date(Date.now() + 3600 * 1000),
        },
    });
});

app.post("/auth/login", (req, res) => {
    const { username, password } = req.body;
    const user = users.find((u) => u.username === username);
    if (user) {
        res.status(200).json({
            success: true,
            id: user.id,
            accessToken: {
                value: "mockAccessToken",
                expirationDate: new Date(Date.now() + 3600 * 1000),
            },
        });
    } else {
        res.status(401).json({
            success: false,
            error: "Invalid credentials",
        });
    }
});

app.post("/auth/token/refresh", (req, res) => {
    const user = users.find((u) => u.username === "testuser");
    if (user) {
        res.status(200).json({
            success: true,
            id: user.id,
            accessToken: {
                value: "mockAccessToken",
                expirationDate: new Date(Date.now() + 3600 * 1000),
            },
        });
    } else {
        res.status(401).json({
            success: false,
            error: "Invalid credentials",
        });
    }
});

// @ts-ignore
app.get('/users/:id', (req, res) => {
    const userId = req.params.id;
    const perPage = parseInt(req.query.per_page as string) || 10;
    const next = req.query.next as string;

    // Find the user
    const user = users.find((u) => u.id === userId);
    if (!user) {
        return res.status(404).json({ message: 'User not found' });
    }

    let userPosts = user.posts!.data.sort(
        (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    );

    // Filter posts based on 'next' timestamp
    if (next) {
        const nextDate = new Date(next);
        if (isNaN(nextDate.getTime())) {
            return res.status(400).json({ message: "Invalid 'next' timestamp" });
        }
        userPosts = userPosts.filter((post) => new Date(post.createdAt) < nextDate);
    }

    const paginatedPosts = userPosts.slice(0, perPage);
    const hasNextPage = userPosts.length > perPage;
    const nextPage = hasNextPage ? paginatedPosts[paginatedPosts.length - 1].createdAt : null;

    // Exclude the full posts from the user object to avoid redundancy
    const userWithoutPosts = { ...user, posts: undefined };

    // Prepare the paginated posts response
    const postsResponse = {
        data: paginatedPosts,
        hasNextPage: hasNextPage,
        perPage: perPage,
        next: nextPage,
    };

    res.json({
        ...userWithoutPosts,
        posts: postsResponse,
    });
});

app.put("/users/:id", (req, res) => {
    res.json({ message: "User updated successfully" });
});

app.delete("/users/:id", (req, res) => {
    res.json({ message: "User deleted successfully" });
});

// @ts-ignore
app.get('/users/:userId/posts', (req, res) => {
    const userId = req.params.userId;
    const perPage = parseInt(req.query.per_page as string) || 10;
    const next = req.query.next as string;

    // Find the user
    const user = users.find((u) => u.id === userId);
    if (!user) {
        return res.status(404).json({ message: 'User not found' });
    }

    // Get the user's posts sorted by createdAt descending
    let userPosts = user.posts!.data.sort(
        (a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime()
    );

    // Filter posts based on 'next' timestamp
    if (next) {
        const nextDate = new Date(next);
        if (isNaN(nextDate.getTime())) {
            return res.status(400).json({ message: "Invalid 'next' timestamp" });
        }
        userPosts = userPosts.filter((post) => new Date(post.createdAt) < nextDate);
    }

    const paginatedPosts = userPosts.slice(0, perPage);
    const hasNextPage = userPosts.length > perPage;
    const nextPage = hasNextPage ? paginatedPosts[paginatedPosts.length - 1].createdAt : null;

    // Prepare the paginated posts response
    const postsResponse = {
        data: paginatedPosts,
        hasNextPage: hasNextPage,
        perPage: perPage,
        next: nextPage,
    };

    res.json(postsResponse);
});

// @ts-ignore
app.get("/users/:userId/feed", (req, res) => {
    const perPage = parseInt(req.query.per_page as string) || 10;
    const next = req.query.next as string;

    let nextDate: Date | null = null;
    if (next) {
        nextDate = new Date(next);
        if (isNaN(nextDate.getTime())) {
            return res.status(400).json({ message: "Invalid 'next' timestamp" });
        }
    }

    // Sort posts in descending order of createdAt
    let filteredPosts = posts.sort((a, b) => new Date(b.createdAt).getTime() - new Date(a.createdAt).getTime());

    // Filter posts based on 'next' timestamp
    if (nextDate) {
        filteredPosts = filteredPosts.filter((post) => new Date(post.createdAt) < nextDate!);
    }

    const paginatedPosts = filteredPosts.slice(0, perPage);
    const hasNextPage = filteredPosts.length > perPage;
    const nextPage = hasNextPage ? paginatedPosts[paginatedPosts.length - 1].createdAt : null;

    res.json({
        data: paginatedPosts,
        hasNextPage: hasNextPage,
        perPage: perPage,
        next: nextPage,
    });
});

// Post Endpoints
app.get("/posts/:id", (req, res) => {
    const post = posts.find((p) => p.id === req.params.id);
    if (post) {
        res.json(post);
    } else {
        res.status(404).json({ message: "Post not found" });
    }
});

app.post("/users/:userId/posts", (req, res) => {
    // Simulate creating a post
    res.status(201).json({ message: "Post created successfully" });
});

app.put("/users/:userId/posts/:postId", (req, res) => {
    // Simulate updating a post
    res.json({ message: "Post updated successfully" });
});

app.delete("/users/:userId/posts/:postId", (req, res) => {
    // Simulate deleting a post
    res.json({ message: "Post deleted successfully" });
});

// Interaction Endpoints
app.post("/users/:userId/likes/posts/:postId", (req, res) => {
    // Simulate liking a post
    res.status(201).json({ message: "Post liked successfully" });
});

app.delete("/users/:userId/likes/posts/:postId", (req, res) => {
    // Simulate unliking a post
    res.status(204).send();
});

app.post("/users/:userId/views/posts/:postId", (req, res) => {
    // Simulate viewing a post
    res.status(201).json({ message: "Post viewed successfully" });
});

// Start the Server
const PORT = 8090;
app.listen(PORT, () => {
    console.log(`Mock API server is running at http://localhost:${PORT}`);
});
