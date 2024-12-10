# Welcome to the Seelen UI Project

Welcome to the Seelen UI project! This guide will help you get started with the codebase and understand its structure.

## Languages Used
This project utilizes the following languages:
- **Rust**
- **TypeScript**
- **PowerShell** (in special cases)

## Getting Started
To run this project, follow these steps:

1. [Install Rust](https://www.rust-lang.org/tools/install).
2. Run the following commands:

```bash
npm install && npm run dev
```

This will set up the project similarly to any other Node.js project, with the added step of installing Rust first.

## Architecture

### Views Architecture

The `src\apps` folder contains views that follow Hexagonal Architecture. Each folder in `src\apps` represents a view (excluding shared). These views are independent web pages bundled with `esbuild`. While any technology or library can be used in a view, most are based on `React` and `Redux`.

#### Shared Folder
Following Hexagonal Architecture, the `shared` folder contains utilities, schemas, and other shared resources used across multiple views.

### Background Architecture

The `src\background` folder does not follow a specific architecture but is based on Events Architecture.

### Hierarchical Locking Order

To prevent deadlocks in the application, all threads must follow the "Hierarchical Locking Order" when acquiring resources:

1. **CLI**: Acquire any locks related to the command-line interface first.
2. **DATA**: Next, acquire locks related to data access or shared data structures.
3. **EVENT**: Finally, acquire locks related to hook or event management.

This order must be respected in all threads to avoid circular wait conditions and ensure safe concurrency.

### Splitting the Codebase - Working with `slu-lib` (Seelen UI Library)

When working with the `slu-lib` library and making changes, follow these steps to configure your development environment. This will allow you to test your changes locally without impacting the main repository.

---

#### 1. Clone the `slu-lib` Repository
Ensure you have a local copy of the `slu-lib` codebase. If you don’t already have it, clone the repository using the following command:

~~~
git clone <slu-lib-repository-url>
cd slu-lib
~~~

---

#### 2. Create a Branch for Your Changes
Create a new branch to implement the necessary changes in the library:

~~~
git checkout -b <your-branch-name>
~~~

Example:
~~~
git checkout -b feature/ui-update
~~~

Make the required changes and commit your modifications:

~~~
git add .
git commit -m "Description of changes made to slu-lib"
~~~

---

#### 3. Option 1, Configure `slu-lib` as a Local Dependency - with remote Repository
To allow the main project to use your modified version of `slu-lib`, update its dependency configuration file.

#### For Rust Projects (`Cargo.toml`):
In the `Cargo.toml` file of the main project, replace the `slu-lib` dependency with your local branch’s Git URL:

~~~
[dependencies]
slu-lib = { git = "<slu-lib-repository-url>", branch = "<your-branch-name>" }
~~~

Example:
~~~
[dependencies]
slu-lib = { git = "https://github.com/your-org/slu-lib", branch = "feature/ui-update" }
~~~

#### For JavaScript Projects (`package.json`):
In the `package.json` file of the main project, update the `slu-lib` dependency to point to your Git branch:

~~~
"dependencies": {
  "slu-lib": "git+https://<slu-lib-repository-url>#<your-branch-name>"
}
~~~

Example:
~~~
"dependencies": {
  "slu-lib": "git+https://github.com/your-org/slu-lib#feature/ui-update"
}
~~~

#### 3. Option 2, Configure `slu-lib` as a Local Dependency - local files
To allow the main project to use your modified version of `slu-lib`, update its dependency configuration file.

#### For Rust Projects (`Cargo.toml`):
In the `Cargo.toml` file of the main project, replace the `slu-lib` dependency with your local path:

~~~
[dependencies.seelen-core]
path = "<path>"                               # for local development
~~~

Example:
~~~
[dependencies.seelen-core]
path = "../slu-lib"                               # for local development
~~~

#### For JavaScript Projects (`package.json`):
In the `package.json` file of the main project, update the `slu-lib` dependency to point to your npm slu home:

~~~
"dependencies": {
  "@seelen-ui/lib": "file:<path>",
}
~~~

slu must be built before npm reference can be made! The build script after deno installation...
~~~
winget install DenoLand.Deno
~~~
can be with the following scripts:
~~~
deno run -A .\scripts\build_npm.ts
~~~

Example:
~~~
"dependencies": {
  "@seelen-ui/lib": "file:../slu-lib/npm",
}
~~~

alternatively, you can run from the main project library: 
~~~
npm install <path>
~~~

Example> 
~~~
npm install ../slu-lib/npm
~~~

**Important:** These changes should only be used for local development and must not be committed to the repository.

---

#### 4. Test and Iterate
- After linking the modified version of `slu-lib`, test the main project to ensure the changes work as expected.
- If further modifications are needed, return to the `slu-lib` repository, make changes, and commit them.

---

#### 5. Finalizing Changes
Once the changes in `slu-lib` are complete:
1. Push your branch to the remote repository:
   ~~~
   git push origin <your-branch-name>
   ~~~
2. Create a pull request to merge your branch into the main `slu-lib` repository.
3. Once the pull request is merged, update the `slu-lib` dependency in the main project to use the new version.
---
