# Quick Start

This guide shows the basic INFYNON workflow.

## 1. Install INFYNON

```bash
npm i -g infynon
```

## 2. Check package safety before install

```bash
infynon pkg npm install axios
```

## 3. Scan your dependency file

```bash
infynon pkg scan --pkg-file package-lock.json
```

## 4. Create a workspace

```bash
infynon workspace create my-project --mutate --folder-name app --path D:/Codeverse/my-project --default
```

## 5. Create a task

```bash
infynon task create task-review-auth --mutate --workspace my-project --prompt "Review auth flow for risky changes."
```

## 6. Launch an agent

```bash
infynon coding claude --cwd D:/Codeverse/my-project
```

## 7. Record the result

```bash
infynon task complete task-review-auth --mutate --result "Review completed."
```
