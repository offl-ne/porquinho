# Porquinho

A piggy-bank-like CLI tool that helps you control your expenses!

![image](https://user-images.githubusercontent.com/38900226/179670363-87ed0001-b3da-4206-a145-cd9066cb4f38.png)

## Installation

Clone the repository and compile it from source.

```sh
git clone https://github.com/vrmiguel/porquinho
cargo install --path porquinho
```

## Usage

Putting money in the piggy:

```sh
porquinho put 42.10 'Description of this transaction'
```

Taking money from the piggy:

```sh
porquinho take 100 'Description of this transaction'
```

Check the status of your balance:

```sh
porquinho status
```
