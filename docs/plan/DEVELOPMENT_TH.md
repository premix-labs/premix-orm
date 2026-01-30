# Master Flowplan: Premix ORM (Ultimate Edition - ภาษาไทย)

## แนวคิดโครงการ
**คอนเซ็ปต์:** "จอกศักดิ์สิทธิ์" แห่งวงการ Rust ORM
**สโลแกน:** "เขียน Rust รันแบบ Optimized SQL"
**สถานะ:** Alpha / Pre-release (v0.x)

## 1. ปรัชญาหลัก (5 เสาหลักแห่งสุดยอด ORM)

### 1. นามธรรมที่ไร้ส่วนเกิน (The "Zero-Overhead" Abstraction) - เร็วที่สุด
> "โค้ดของคุณควรเร็วเท่ากับการเขียน Raw SQL ด้วยมือ"
- **แนวทาง:** สร้าง SQL ตั้งแต่ตอนคอมไพล์ รันไทม์ทำหน้าที่แค่ส่ง SQL ที่เตรียมไว้แล้ว
- **Optimization:** Smart Configuration จูน Connection Pool อัตโนมัติตามสภาพแวดล้อม (เช่น Server vs Serverless)

### 2. ตรงกับวิธีคิดของคน (The "Mental Model Match") - ง่ายที่สุด
> "โค้ดควรหน้าตาเหมือนวิธีที่คุณคิด ไม่ใช่วิธีที่ Database เก็บข้อมูล"
- **แนวทาง:** เชื่อมโลก Object และ Table อย่างแนบเนียน
- **เป้าหมาย:** API ใช้งานง่าย และประสบการณ์การเขียน Test ที่ลื่นไหล

### 3. ความปลอดภัยที่ไม่มีวันล้มเหลว (The "Impossible-to-Fail" Safety) - เสถียรที่สุด
> "ถ้าคอมไพล์ผ่าน มันต้องรันได้โดยไม่มี SQL Error"
- **แนวทาง:** ย้าย Error ไปไว้ตอนคอมไพล์
- **Safety Rails:** Destructive Guards ป้องกันการลบข้อมูลทั้งตารางโดยไม่ตั้งใจ (ต้องยืนยันเสมอ)

### 4. ความโปร่งใสแบบกล่องแก้ว (The "Glass Box" Transparency) - โปร่งใส
> "เวทมนตร์น่ะดี แต่มนต์ดำไม่ดี"
- **แนวทาง:** เปิดเผย SQL ที่ถูกสร้างขึ้นได้เสมอ
- **Security:** Sensitive Data Masking ปิดบังข้อมูลสำคัญใน Log (***) เพื่อความปลอดภัย (PDPA/GDPR)

### 5. ทางหนีที่ไหลลื่น (The "Graceful Escape Hatch") - ยืดหยุ่น
> "เรื่องง่ายต้องทำได้ง่าย เรื่องยากต้องเป็นไปได้"
- **แนวทาง:** ผสมผสาน Raw SQL ได้อย่างลื่นไหล
- **ฟีเจอร์:** Arbitrary Struct Mapping รองรับการ Map ผลลัพธ์เข้า Struct อิสระสำหรับทำ Report ที่ซับซ้อน

---

## 2. แผนการพัฒนา (Development Flowplan)

### เฟส 0: การติดตั้งและสถาปัตยกรรม - ✅ เสร็จสมบูรณ์
**ภารกิจ:** วางรากฐานโครงสร้างโปรเจกต์ให้ขยายได้
- [x] เริ่มต้นโปรเจกต์ & แยกโมดูล
- [x] มาโครพื้นฐาน `#[derive(Model)]`

### เฟส 1: กลไก CRUD - ✅ เสร็จสมบูรณ์
**ภารกิจ:** บันทึกและดึงข้อมูลพื้นฐาน
- [x] ระบบจับคู่ Type
- [x] การเชื่อมต่อ Database

### เฟส 2: เวทมนตร์แห่ง Migration - ✅ เสร็จสมบูรณ์
**ภารกิจ:** อัปเดต Schema อัตโนมัติ
- [x] Schema introspection & Diff engine
- [x] คำสั่ง `Premix::sync()`
  - SQLite v1: diff ตาราง/คอลัมน์/ชนิด/nullable/pk/indexes/foreign keys และสร้าง SQL
  - Postgres v1: diff ตาราง/คอลัมน์/ชนิด/nullable/pk/indexes/foreign keys และสร้าง SQL

### เฟส 3: ความสัมพันธ์และการปรับแต่ง - ✅ เสร็จสมบูรณ์
**ภารกิจ:** แก้ปัญหา N+1 Query
- [x] มาโครความสัมพันธ์ (`has_many`, `belongs_to`)
- [x] Eager loading (`.include()`) แบบ O(1) query strategy
  - รองรับทั้ง has_many และ belongs_to

### เฟส 4: ประสบการณ์นักพัฒนา (DX) - ✅ เสร็จสมบูรณ์
**ภารกิจ:** ทำให้ ORM ใช้งานได้จริง ทดสอบง่าย และย้ายมาใช้ง่าย
- [x] CLI Tool, เอกสาร และตัวอย่างการใช้งาน
- [x] Macro error handling ที่ชี้ตำแหน่งถูกต้อง
- [x] เอกสาร Glass Box (macro expansion + SQL flow)
- [x] คู่มือ Performance Tuning (prepared statements, fast/static paths)
- [x] Test Utilities (ของใหม่):
  - [x] Transactional Tests: Rollback อัตโนมัติหลังจบแต่ละ Test Case
  - [x] ตัวช่วย Mock Database
- [x] Database Scaffolding (ของใหม่):
  - [x] `premix-cli scaffold`: สร้าง Struct Rust จาก Database เดิม (Reverse Engineering)
- [x] Framework Integrations (ของใหม่):
  - [x] Crate เสริมอย่างเป็นทางการ: `premix-axum`, `premix-actix`

### เฟส 5: มาตรฐานระดับองค์กร - ✅ เสร็จสมบูรณ์
**ภารกิจ:** รองรับความซับซ้อน ความปลอดภัย และการทำ Report
- [x] Observability (`tracing`)
- [x] ACID transactions & lifecycle hooks
- [x] Optimistic locking & validation
- [x] Arbitrary Struct Mapping (ของใหม่):
  - [x] `Premix::raw("...").fetch_as::<ReportStruct>()`
- [x] Sensitive Data Masking (ของใหม่):
  - [x] Attribute `#[premix(sensitive)]` เพื่อปิดบังข้อมูลใน Log
- [x] Smart Configuration (ของใหม่):
  - [x] ระบบจูน Pool Config อัตโนมัติตามสภาพแวดล้อม

### เฟส 6: ความอเนกประสงค์ - ✅ เสร็จสมบูรณ์
**ภารกิจ:** ลบข้อจำกัด และเพิ่มความปลอดภัย
- [x] Multi-database (SQLite, Postgres, MySQL), Soft deletes, Bulk operations
- [x] Destructive Guards (ของใหม่):
  - [x] ป้องกัน `delete_all()` ถ้าไม่มี `.filter()` หรือ `.allow_unsafe()`

### เฟส 7: DevOps และ Versioned Migrations - ✅ เสร็จสมบูรณ์
**ภารกิจ:** รองรับการทำงานเป็นทีม (Release Readiness) เป้าหมาย: v1.0.0 RC
- [x] ระบบ `premix-cli migrate` สมบูรณ์แบบ
- [x] ไฟล์ Migration แบบระบุเวอร์ชัน (`YYYYMMDD_name.sql`)

### เฟส 8: การขยายตัว (The Scale) - 📝 อยู่ในแผน
**ภารกิจ:** รองรับระบบขนาดใหญ่ (High Availability) และการวัดผล เป้าหมาย: v1.1.0
- [ ] แยก Read/Write (Primary + Replicas)
- [ ] Multi-tenancy Support
- [ ] Metrics Collection (ของใหม่):
  - [x] ส่งออกค่า Pool stats (idle/active) และ Query latency สำหรับทำ Dashboard (Prometheus/Grafana)

### เฟส 9: ความสัมพันธ์ขั้นสูง - ⏳ เลื่อนออกไปก่อน
**ภารกิจ:** รองรับ Model ซับซ้อนเฉพาะทาง
- [ ] Polymorphic relations

### เฟส 10: การรองรับระบบเก่า - 📝 อยู่ในแผน
**ภารกิจ:** รองรับ Brownfield projects
- [ ] Composite Primary Keys
- [ ] Custom Postgres types

---

## 3. ชุดเครื่องมืออัตโนมัติ (Developer Automation Suite)
ชุดสคริปต์มาตรฐานอยู่ในโฟลเดอร์ `scripts/` (dev, test, ci, bench, release)

---

## 4. ความเสี่ยงทางวิศวกรรม (Engineering Risks)
- **Compile Time:** แก้ไขด้วย Lean Codegen
- **Error Messages:** แก้ไขด้วย `syn::spanned`
- **Async Traits:** แก้ไขด้วย `BoxFutures`
- **ข้อมูลหายโดยไม่ตั้งใจ:** แก้ไขด้วย Destructive Guards
- **ข้อมูลหลุดใน Log:** แก้ไขด้วย Sensitive Data Masking
