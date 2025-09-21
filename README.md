# CLIERP

> **CLI-based Enterprise Resource Planning System**

CLIERP는 전통적인 ERP(Enterprise Resource Planning) 시스템의 기능을 CLI(Command Line Interface) 환경으로 제공하는 혁신적인 융합 시스템입니다.

## 🚀 핵심 특징

- **순수 CLI**: GUI 없이 터미널 환경에서 모든 ERP 기능 사용
- **고성능**: Rust 언어로 구현된 안전하고 빠른 시스템
- **모듈화**: HR, Finance, Inventory, CRM 모듈의 독립적 관리
- **확장성**: 플러그인 시스템을 통한 기능 확장
- **자동화**: CLI 특성을 활용한 배치 처리 및 스크립팅

## 📋 주요 모듈

### 🏢 HR (인사관리)
```bash
clierp hr employee add --name "김철수" --dept "개발팀"
clierp hr attendance checkin --employee-id 123
clierp hr payroll calculate --month 2024-09
```

### 💰 Finance (재무관리)
```bash
clierp fin account create --name "매출" --type "revenue"
clierp fin transaction add --account "매출" --amount 1000000
clierp fin report income-statement --period "2024-09"
```

### 📦 Inventory (재고관리)
```bash
clierp inv product add --name "노트북" --sku "LT001"
clierp inv stock update --sku "LT001" --quantity 50
clierp inv order create --supplier "삼성" --items "LT001:10"
```

### 👥 CRM (고객관리)
```bash
clierp crm customer add --name "ABC기업" --type "기업"
clierp crm lead add --customer-id 123 --value 5000000
clierp crm deal create --lead-id 456 --stage "제안"
```

## 🛠️ 기술 스택

- **언어**: Rust
- **CLI 프레임워크**: clap
- **데이터베이스**: SQLite (개발), PostgreSQL (운영)
- **ORM**: Diesel
- **로깅**: tracing

## 📚 문서

- [시스템 아키텍처](docs/architecture.md)
- [데이터베이스 설계](docs/database_design.md)
- [개발 로드맵](docs/development_roadmap.md)

## 🏗️ 개발 현황

현재 개발 초기 단계로, 다음과 같은 로드맵으로 진행됩니다:

- **Phase 1**: 기반 구조 (1-2개월) - 🚧 진행 예정
- **Phase 2**: 핵심 모듈 (2-3개월)
- **Phase 3**: 확장 모듈 (2-3개월)
- **Phase 4**: 고도화 (1-2개월)

## 🚀 설치 및 실행

```bash
# 저장소 클론
git clone https://github.com/YOUR_USERNAME/CLIERP.git
cd CLIERP

# 빌드 및 실행
cargo build --release
cargo run -- --help
```

## 🤝 기여하기

CLIERP는 오픈소스 프로젝트입니다. 기여를 환영합니다!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📄 라이선스

이 프로젝트는 MIT 라이선스 하에 배포됩니다. 자세한 내용은 [LICENSE](LICENSE) 파일을 참조하세요.

## 📞 연락처

프로젝트 관련 문의나 제안이 있으시면 이슈를 생성해 주세요.

---

*CLIERP - CLI로 만나는 새로운 ERP 경험*