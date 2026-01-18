# แผนงานหลัก: Premix ORM

## คอนเซปต์โครงการ
**คอนเซปต์:** "จอกศักดิ์สิทธิ์" แห่งวงการ Rust ORM (The Holy Grail of Rust ORMs)  
**สโลแกน:** "เขียนแบบ Rust รันแบบ Optimized SQL"  
**สถานะ:** ต้นแบบงานวิจัย (Research prototype); ยังไม่แนะนำให้ใช้ในระบบจริง (Production)

## 1. ปรัชญาหลัก (5 เสาหลักแห่งสุดยอด ORM)

### 1. นามธรรมที่ไร้ส่วนเกิน (The "Zero-Overhead" Abstraction) - เร็วที่สุด
> "โค้ดของคุณควรเร็วเท่ากับการเขียน Raw SQL ด้วยมือ"
- **แนวทางของ Premix:** ใช้ Rust macros สร้าง SQL builder ตั้งแต่คอมไพล์ และประกอบ SQL string ตอนรันไทม์เพื่อส่งให้ `sqlx`.
- **เป้าหมาย:** Benchmark ตอนรันไทม์ต้องเท่ากับ `sqlx` แบบดิบ (Overhead 0%).
- **สถานะปัจจุบัน:** ประสิทธิภาพใกล้เคียงกับ Raw SQL ในการทดสอบ แต่ยังมีการประกอบ SQL บางส่วนตอนรันไทม์.

### 2. ตรงกับวิธีคิดของคน (The "Mental Model Match") - ง่ายที่สุด
> "โค้ดควรหน้าตาเหมือนวิธีที่คุณคิด ไม่ใช่วิธีที่ Database เก็บข้อมูล"
- **แนวทางของ Premix:** เชื่อมโลกของ Object (`User`) กับโลกของตาราง (`users`) อย่างแนบเนียน.
  - **การตั้งชื่อ:** `User` -> `users` อัตโนมัติ.
  - **Active Record:** `user.save()`, `User::find(1)`.
  - **Auto Migration:** แก้ Struct แล้วฐานข้อมูลอัปเดตตาม.

### 3. ความปลอดภัยที่ไม่มีวันล้มเหลว (The "Impossible-to-Fail" Safety) - เสถียรที่สุด
> "ถ้าคอมไพล์ผ่าน มันต้องรันได้โดยไม่มี SQL Error"
- **แนวทางของ Premix:** ย้าย Error มาไว้ที่เวลาคอมไพล์.
  - พิมพ์ชื่อฟิลด์ผิด -> Compile error.
  - ประเภทข้อมูลไม่ตรง -> Compile error.
- **เป้าหมาย:** กำจัด "เรื่องเซอร์ไพรส์ตอนรันไทม์" ให้หมดไป.
- **สถานะปัจจุบัน:** ฟิลด์ของ Model ตรวจสอบได้ตอนคอมไพล์ แต่การกรองแบบ String ยังพลาดตอนรันไทม์ได้.

### 4. ความโปร่งใสแบบกล่องแก้ว (The "Glass Box" Transparency) - โปร่งใส
> "เวทมนตร์น่ะดี แต่มนต์ดำน่ะไม่ดี"
- **แนวทางของ Premix:** เปิดเผย SQL ที่ถูกสร้างขึ้นผ่าน `to_sql()` หรือ helper ที่เกี่ยวข้อง.
- **เป้าหมาย:** นักพัฒนาต้องยังรู้สึกว่า "ควบคุม" SQL ได้.
- **สถานะปัจจุบัน:** Query builder มี `to_sql()`, `to_update_sql()`, `to_delete_sql()`.

### 5. ทางหนีทีไล่ที่สวยงาม (The "Graceful Escape Hatch") - ยืดหยุ่น
> "เรื่องง่ายต้องทำได้ง่าย เรื่องยากต้องเป็นไปได้"
- **แนวทางของ Premix:** อนุญาตให้ผสม Raw SQL กับ ORM ได้อย่างลื่นไหลในเคสซับซ้อน.
- **ตัวอย่าง:** `User::raw_sql("SELECT * FROM ...").fetch_all()`
- **สถานะปัจจุบัน:** `Model::raw_sql(...)` พร้อมใช้งานสำหรับ map ผลลัพธ์กลับเป็น Model.

ดูสถานะแบบละเอียดได้ที่ `docs/PHILOSOPHY_CHECKLIST.md`.

---

## 2. แผนการพัฒนา (Development Flowplan)

### เฟส 0: การติดตั้งและสถาปัตยกรรม (Setup and Architecture) - เสร็จสมบูรณ์
**ภารกิจ:** วางรากฐานโครงสร้างโปรเจกต์ให้รองรับการขยายตัว

- [x] เริ่มต้นโปรเจกต์ (Cargo workspace).
- [x] แยกโมดูล (`premix-core` สำหรับ runtime, `premix-macros` สำหรับ compiler).
- [x] มาโครพื้นฐาน `#[derive(Model)]`.
- [x] การสร้างชื่อตารางอัตโนมัติ (`User` -> `users`).
- [x] การดึงชื่อคอลัมน์จาก struct fields.

**Tech Stack:**
- Database driver: `sqlx`
- Macro engine: `syn`, `quote`, `proc-macro2`
- Runtime: `tokio`

### เฟส 1: กลไก CRUD (The CRUD Engine) - เสร็จสมบูรณ์
**ภารกิจ:** ทำให้การบันทึกและดึงข้อมูลพื้นฐานทำงานได้

- [x] ระบบจับคู่ Type (`i32` -> `INTEGER`, `String` -> `TEXT`).
- [x] เชื่อมต่อกับ `sqlx`.

**Milestone 1:** `User::new().save().await` บันทึกข้อมูลลงฐานข้อมูลจริงได้.

### เฟส 2: เวทมนตร์แห่ง Migration (The Migration Magic) - เสร็จสมบูรณ์
**ภารกิจ:** ไม่ต้องเขียน SQL เองเมื่อมีการแก้ไขโครงสร้างข้อมูล

- [x] การตรวจสอบโครงสร้าง (Schema introspection).
- [x] ระบบเปรียบเทียบความต่าง (Diff engine) ระหว่าง struct และ database.
- [x] `Premix::sync()` เพื่อรัน `CREATE TABLE` หรือ `ALTER TABLE`.

**Milestone 2:** เพิ่ม field ใน struct แล้วกด sync ฐานข้อมูลจะอัปเดตตามทันที.

### เฟส 3: ความสัมพันธ์และการปรับแต่ง (Relations and Optimization) - เสร็จสมบูรณ์
**ภารกิจ:** แก้ปัญหา N+1 Query และทำให้ Query แบบ Fluent ใช้งานได้

- [x] มาโครความสัมพันธ์: `#[has_many]`, `#[belongs_to]`.
- [x] Query builder: `Model::find()`, `.filter()`, `.limit()`, `.offset()`.
- [x] การโหลดล่วงหน้า (Eager loading) ด้วย `.include("posts")`.
- [x] `#[premix(ignore)]` สำหรับ field ที่ไม่ใช่คอลัมน์ในตาราง.

**Milestone 3:** ดึง User 100 คนพร้อม Posts ได้โดยไม่เกิด N+1 Query.

### เฟส 4: ประสบการณ์นักพัฒนา (Developer Experience) - เสร็จสมบูรณ์
**ภารกิจ:** ทำให้ ORM ใช้งานได้จริงในโปรเจกต์

- [x] เครื่องมือ CLI (`premix-cli`).
- [x] เอกสารคู่มือและตัวอย่างโปรเจกต์.
- [x] การจัดการ Error ของ Macro ด้วย `syn::Error`.

**Milestone 4:** ระบบนิเวศครบทั้งฝั่ง Runtime และ DX.

### เฟส 5: มาตรฐานระดับองค์กร (Enterprise Standard) - เสร็จสมบูรณ์
**ภารกิจ:** รองรับความซับซ้อนของโลกจริง

- [x] Observability ผ่าน `tracing`.
- [x] ธุรกรรมแบบ ACID ด้วย `pool.begin()`.
- [x] Lifecycle hooks (`before_save`, `after_save`).
- [x] Optimistic locking ผ่าน `update()`.
- [x] Validation ผ่าน `validate()`.

**Milestone 5:** พร้อมสำหรับระบบขนาดใหญ่.

### เฟส 6: ความอเนกประสงค์ (The Versatility) - เสร็จสมบูรณ์
**ภารกิจ:** ลบข้อจำกัดและรองรับหลายฐานข้อมูล

- [x] สถาปัตยกรรม Multi-database ผ่าน `SqlDialect`.
- [x] `Model<DB>` และ `QueryBuilder<DB>` แบบ Generic.
- [x] รองรับ SQLite, PostgreSQL, MySQL (ผ่าน feature flags).
- [x] Soft deletes (`deleted_at`).
- [x] Bulk operations (`QueryBuilder::update`, `QueryBuilder::delete`).
- [x] รองรับ JSON/JSONB ผ่าน `serde_json`.

**Milestone 6:** รองรับ Multi-DB และ Bulk operations เรียบร้อยแล้ว.

### เฟส 7: DevOps และ Versioned Migrations - เสร็จสมบูรณ์
**ภารกิจ:** รองรับการทำงานเป็นทีมและการปล่อยของ (Release)

- [x] ชุดคำสั่ง `premix-cli migrate`.
- [x] คำสั่ง `create`, `up` สำหรับ migrations.
- [x] การติดตามสถานะ Migration ผ่านตาราง `_premix_migrations`.

**Milestone 7:** ระบบ Migration สำหรับ Production พร้อมใช้งาน.

### เฟส 8: การขยายตัว (The Scale) - อยู่ในแผน
**ภารกิจ:** ความพร้อมใช้งานสูงสำหรับระบบขนาดใหญ่

- [ ] แยกการอ่าน/เขียน (Primary + Replicas).
- [ ] ตัวจัดการ Connection สำหรับระบบ Multi-tenancy.

### เฟส 9: ความสัมพันธ์ขั้นสูง (Advanced Relations) - เลื่อนออกไปก่อน
**ภารกิจ:** รองรับการออกแบบข้อมูลที่ซับซ้อน/เฉพาะทาง

- [ ] ความสัมพันธ์แบบ Polymorphic.
- [ ] การประกาศ Schema แบบ Declarative.

### เฟส 10: การรองรับระบบเก่า (Legacy Support) - อยู่ในแผน
**ภารกิจ:** รองรับโปรเจกต์ที่มีอยู่เดิม (Brownfield)

- [ ] Primary key แบบผสม (Composite primary keys).
- [ ] Custom Types และ Domains ของ Postgres.

### อนาคต (Futurism) - อยู่ในแผน
**ภารกิจ:** เตรียมพร้อมสำหรับเทรนด์ในระยะยาว

- [ ] Vector types และ Semantic search.
- [ ] รองรับ Edge/Wasm targets.
- [ ] การซิงค์แบบ Local-first (CRDTs).
- [ ] ระบบปรับจูนตัวเองอัตโนมัติ (Adaptive self-optimization).

---

## 3. ชุดเครื่องมืออัตโนมัติสำหรับนักพัฒนา (Developer Automation Suite)

สคริปต์ทั้งหมดอยู่ที่ `scripts/`:

### `scripts/dev` (การพัฒนาประจำวัน)
- `run_fmt.ps1`: จัดรูปแบบโค้ดและแก้คำเตือน clippy.
- `run_clean.ps1`: ล้างไฟล์ build และ database เก่าๆ.
- `gen_docs.ps1`: สร้างเอกสาร rustdoc และ mdBook.

### `scripts/test` (การตรวจสอบความถูกต้อง)
- `test_quick.ps1`: Smoke test (Build + รันแอปพื้นฐาน).
- `test_examples.ps1`: รันตัวอย่างแอปทั้งหมด.
- `test_migration.ps1`: ทดสอบระบบ Migration แบบ E2E.

### `scripts/ci` (การประกันคุณภาพ)
- `check_all.ps1`: ตรวจสอบทั้ง Workspace (Build, Test, Clippy, Format).
- `check_audit.ps1`: สแกนความปลอดภัยผ่าน `cargo audit`.
- `check_coverage.ps1`: เช็คความครอบคลุมของโค้ดผ่าน `cargo tarpaulin`.

### `scripts/bench` (ประสิทธิภาพ)
- `bench_orm.ps1`: Benchmark เทียบ SQLite กับ ORM เจ้าอื่น.
- `bench_io.ps1`: Benchmark I/O ของ Postgres.

### `scripts/release` (การปล่อยซอฟต์แวร์)
- `run_publish.ps1`: ปล่อยแพ็กเกจขึ้น crates.io.

---

## 4. แผนภาพสถาปัตยกรรม (แบบจำลองความคิด)

### ขั้นตอนการทำงาน
1. **เวลาคอมไพล์:** ผู้ใช้เขียน Rust -> มาโครสร้าง SQL ที่ปรับแต่งแล้ว.
2. **เวลารัน:** แอปประมวลผล SQL ที่ถูกสร้างไว้ผ่าน `sqlx`.

---

## 5. ความเสี่ยงทางวิศวกรรม (Engineering Risks)

### 1. เวลาคอมไพล์ที่บวมขึ้น (Compile Time Explosion)
- **ความเสี่ยง:** มาโครทำงานหนักทำให้การ Build ช้าลง.
- **ทางแก้:** รักษา codegen ให้เล็กที่สุดและรองรับ incremental compilation.

### 2. ความซับซ้อนของข้อความ Error (Error Message Complexity)
- **ความเสี่ยง:** Error ของมาโครชี้ไปผิดที่ หรืออ่านไม่รู้เรื่อง.
- **ทางแก้:** ใช้ `syn::spanned` เพื่อชี้เป้า Error ให้แม่นยำ.

### 3. Async ใน Traits (Async in Traits)
- **ความเสี่ยง:** ข้อจำกัดของ Rust เกี่ยวกับ async trait bounds.
- **ทางแก้:** ออกแบบ ownership ให้ดี และใช้ BoxFutures เมื่อจำเป็น.
