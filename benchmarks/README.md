# 🏁 Premix ORM Benchmarks

This folder contains performance benchmarks comparing **Premix ORM** against **sqlx (Raw SQL)**, **SeaORM**, and **Rbatis**.

## 🚀 How to Run

To run the full suite of benchmarks using [Criterion.rs](https://github.com/bheisler/criterion.rs):

```bash
cd benchmarks
cargo bench
```

This will compile the benchmarks and run them. Be patient, as compiling all ORM dependencies and running statistical analysis takes time.

### Quick Run (Less Accurate)
If you want results faster (fewer samples):

```bash
cargo bench -- --sample-size 10
```

## 📊 What We Measure

We compare these candidates:
1.  **sqlx (raw)**: The baseline speed (manual SQL writing).
2.  **Premix ORM**: Our hero.
3.  **SeaORM**: Popular async ORM.
4.  **Rbatis**: MyBatis style ORM.

Across these scenarios:
- **Insert**: Single row insertion.
- **Select (One)**: Fetching a single row by ID (`find_by_id`).
- **Bulk Select (100)**: Fetching 100 rows and mapping them to Structs.

## 📈 Viewing Reports

After running `cargo bench`, Criterion will generate a detailed HTML report at:

```
premix-orm/target/criterion/report/index.html
```

Open this file in your browser to see beautiful graphs and performance distributions!
