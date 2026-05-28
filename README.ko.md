# WB Slide

가벼운 슬라이드 프레젠테이션 프레임워크. 마크다운을 넣으면 슬라이드가 나옵니다.

## 왜 WB Slide인가?

**실행 파일 하나, 의존성 제로.** 파일 하나만 다운로드하면 끝입니다.
npm도, Python도, Ruby도, 빌드 과정도 필요 없습니다.
프레젠테이션 만드는 데 패키지 매니저를 설치할 필요가 없어야 합니다.

**마크다운 중심.** 슬라이드를 텍스트로 작성합니다. 드래그앤드롭도, 독점 포맷도 없습니다.
Git으로 버전 관리하기 쉽고, diff로 변경사항 확인하기 쉽고, 협업하기 쉽습니다.

**일관된 결과물.** 레이아웃 템플릿을 한 번 정의하면 어디서든 재사용할 수 있습니다.
팀의 모든 발표 자료가 동일한 간격, 동일한 서체, 동일한 구조를 갖습니다.

**AI/LLM 친화적.** 마크다운은 LLM이 가장 잘 다루는 포맷입니다.
AI에게 슬라이드 작성을 요청하고 결과를 `slides.md`에 그대로 붙여넣으면 됩니다.
PowerPoint에 복사/붙여넣기할 필요가 없습니다.

**필요할 때 확장 가능.** 커스텀 레이아웃은 Web Components입니다.
`layouts/` 폴더에 `.js` 파일을 넣으면 됩니다. 스타일은 `styles/`에 넣으면 됩니다.
기본 제공 레이아웃으로 90%는 충분하고, 나머지 10%를 위한 확장이 준비되어 있습니다.

## 설치

```bash
curl -fsSL https://raw.githubusercontent.com/warmblood-kr/wb-slide/main/install.sh | sh
```

또는 [Releases](https://github.com/warmblood-kr/wb-slide/releases)에서 직접 다운로드할 수 있습니다.

macOS (Apple Silicon), Linux (x64), Windows (x64)을 지원합니다.

## 시작하기

```bash
mkdir my-deck && cd my-deck

cat > slides.md << 'EOF'
---
title: 안녕하세요
layout: slide-cover
---

# 안녕하세요

---
layout: slide-feature
heading: 첫 번째 슬라이드
subtitle: wb-slide로 시작하기.
---

**마크다운**이나 HTML로 내용을 작성하세요.

---
layout: slide-section
---

# 감사합니다
EOF

wb-slide show
```

브라우저가 `http://localhost:3030`에서 열립니다. 화살표 키로 이동합니다.

## 명령어

```bash
wb-slide show                          # 발표 시작 (브라우저 열기)
wb-slide show --port 8080              # 포트 지정
wb-slide show --dir path/to/deck       # 다른 디렉토리 지정

wb-slide export                        # export.html로 내보내기
wb-slide export -o presentation.html   # 파일명 지정

wb-slide version                       # 업데이트 확인
wb-slide update                        # 최신 버전으로 자동 업데이트
```

## 단축키

`→` / `Space` 다음 | `←` 이전 | `Home` / `End` 처음/끝 | `F` 전체화면

## 레이아웃

| 레이아웃 | 용도 |
|---------|------|
| `slide-cover` | 표지 (워터마크/푸터 없음) |
| `slide-feature` | 제목 + 부제 + 콘텐츠 |
| `slide-section` | 섹션 구분 |
| `slide-default` | 일반 콘텐츠 |
| `slide-contact` | 연락처 |
| `slide-two-column` | 2단 레이아웃 |
| `slide-image-full` | 전체 이미지 |
| `slide-quote` | 인용문 |

커스텀 레이아웃이 필요하면 `layouts/`에 `.js` 파일을 넣으세요. [docs/layouts.md](docs/layouts.md) 참조.

## 스타일 커스터마이징

`styles/` 디렉토리에 CSS 파일을 넣으면 자동으로 적용됩니다:

```css
/* styles/custom.css */
:root {
  --color-accent: #FF6600;
  --font-family: 'Pretendard', sans-serif;
}
```

## 디렉토리 구조

```
my-deck/
  slides.md        # 슬라이드 내용 (필수)
  styles/          # CSS 오버라이드 (자동 로드)
  layouts/         # 커스텀 레이아웃 (자동 로드)
  assets/          # 이미지, 아이콘 등
```

## 문서

- [슬라이드 포맷](docs/slide-format.md) -- frontmatter, 마크다운, 콘텐츠 규칙
- [레이아웃](docs/layouts.md) -- 빌트인 레이아웃, 커스텀 레이아웃 만들기
- [스타일링](docs/styling.md) -- 테마 변수, CSS 유틸리티, 인쇄/PDF
- [아키텍처](docs/architecture.md) -- 내부 동작 원리

## 라이선스

[MIT](LICENSE)
