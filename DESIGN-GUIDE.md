# 사용자 기능 소개서 Design Guide — Monocle AI (Warmblood)

> PDF 원본: `Monocle AI 사용자 기능 소개서-20260514.pdf` 기반 추출

## Color Palette

| Role | Hex | Usage |
|------|-----|-------|
| Accent Orange | `#FF6B00` | 슬라이드 제목, 섹션 타이틀, 목차 제목 |
| Text Dark | `#2D3436` | 본문 텍스트, 서브타이틀 |
| Text Gray | `#636E72` | 부제, 보조 텍스트 |
| Header Gray | `#B2BEC3` | 우측 상단 "Monocle AI" 워터마크 |
| Navy Dark | `#1B2A4A` | Contact 페이지 제목 |
| White | `#FFFFFF` | 슬라이드 배경 (전체) |
| Light Gray BG | `#F8F9FA` | 스크린샷 배경/그림자 영역 |
| Orange Highlight | `#FF8C33` | 강조 박스 테두리 (크래프트 페이지 등) |

## Typography

- **Primary Font**: Pretendard
  - CDN: `https://cdn.jsdelivr.net/gh/orioncactus/pretendard/dist/web/static/pretendard.css`
  - Fallback: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif
- **슬라이드 제목**: Pretendard Bold, ~2.2rem, Accent Orange
- **서브타이틀/설명**: Pretendard Regular, ~1.1rem, Text Dark
- **우측 상단 워터마크**: Pretendard Light, ~0.9rem, Header Gray
- **하단 로고**: Warmblood 로고 (커스텀 타이포그래피, italic W)
- **페이지 번호**: Regular, ~0.8rem, Text Gray, 우측 하단

## Slide Layout Structure

### 공통 요소 (모든 슬라이드)
- **우측 상단**: "Monocle AI" 텍스트 워터마크 (Header Gray)
- **좌측 하단**: Warmblood 로고 (커스텀 타이포)
- **우측 하단**: 페이지 번호
- **배경**: 흰색 (#FFFFFF)
- **비율**: 16:9 (960×540pt 기준)

### 슬라이드 타입별 레이아웃

#### 1. 커버 (Page 1)
- 중앙 상단: Monocle AI 로고 + "Monocle AI 기능 소개서" 타이틀
- 하단: 노트북 목업 안에 제품 스크린샷
- 워터마크/로고/페이지번호 없음

#### 2. 목차 (Page 2)
- 중앙: "목차" 제목 (Accent Orange, Bold, 대형)
- 아래: 카테고리명 + 불릿 리스트
- 불릿: 기본 원형 (•)

#### 3. 섹션 구분자 (Page 3)
- 화면 정중앙: 섹션명 (Accent Orange, Bold, 대형)
- 최소한의 디자인, 여백 극대화

#### 4. 기능 소개 (Pages 4–14)
- **좌측 상단**: 기능명 제목 (Accent Orange, Bold, ~2.2rem)
- **제목 아래**: 한 줄 설명 (Text Dark, Regular, ~1.1rem)
- **중앙~하단**: 제품 UI 스크린샷 (1~2장)
  - 단일 스크린샷: 중앙 배치, 미세한 그림자
  - 복수 스크린샷: 약간 겹치게 배치 (z-index 레이어링)
- 스크린샷에 주황색 테두리 하이라이트 박스가 있는 경우 있음

#### 5. Contact (Page 15)
- **좌측 상단**: "Contact" (Navy Dark, Bold)
- 아래: 회사명, URL, 이메일, 전화번호 (Text Dark)
- URL은 링크 스타일 (하이라이트 없음, 텍스트만)

## Screenshot Presentation

- **프레임**: 브라우저 윈도우 스타일 (macOS 창 버튼 포함)
- **그림자**: `box-shadow: 0 4px 24px rgba(0,0,0,0.12)`
- **모서리**: `border-radius: 8px`
- **배치**: 단일은 중앙, 복수는 겹침 레이아웃 (좌하단 + 우상단)
- **크기**: 슬라이드 너비의 60~80%

## Asset Inventory

```
assets/
  logos/
    monocle-ai-logo-dark.png    — 어두운 배경용 (눈 일러스트 + 텍스트)
    monocle-ai-logo-white.png   — 밝은 배경용 (원형 + 텍스트, 흰색)
  screenshots/
    cover-laptop-mockup.png     — 커버 페이지 노트북 목업
    multi-vendor.png            — 멀티벤더 모델 선택 UI
    multi-vendor-compare.png    — 멀티벤더 동시 호출 비교 UI
    project.png                 — 프로젝트 (팀 협업) UI
    m365-integration.png        — M365 연동 상세
    m365-input-bar.png          — M365 입력바
    web-search.png              — 웹 자료 활용 UI
    file-attachment.png         — 파일 첨부 UI
    reference-materials.png     — 참고 자료 UI
    custom-models.png           — 커스텀 모델 UI
    craft.png                   — 크래프트 (문서 생성) UI
    image-generation.png        — 이미지 생성 UI
    memory.png                  — 메모리 UI
  icons/
    (M365 관련 아이콘 등)
  pages/
    page-01.png ~ page-15.png   — 원본 페이지 전체 렌더링 (300dpi)
```

## Slidev Migration Notes

### 슬라이드 구조 (15 slides)
| # | 타입 | 제목 | 레이아웃 |
|---|------|------|----------|
| 1 | cover | Monocle AI 기능 소개서 | 커스텀 (로고 + 목업) |
| 2 | toc | 목차 | center + list |
| 3 | section | 사용자용 기능 | center |
| 4 | feature | 멀티 벤더 | title-screenshot |
| 5 | feature | 멀티 벤더 동시 호출 | title-screenshot |
| 6 | feature | 프로젝트 (팀 협업 공간) | title-screenshot |
| 7 | feature | M365 (오피스웨어) 연동 | title-screenshot-icons |
| 8 | feature | 웹 자료 활용 | title-screenshot-overlap |
| 9 | feature | 파일 첨부 | title-screenshot |
| 10 | feature | 참고 자료 | title-screenshot-overlap |
| 11 | feature | 커스텀 모델 | title-screenshot |
| 12 | feature | 크래프트 | title-screenshot |
| 13 | feature | 이미지 생성 | title-screenshot |
| 14 | feature | 메모리 | title-screenshot |
| 15 | contact | Contact | info-list |

### Slidev 커스텀 레이아웃 필요
- `feature`: 주황색 제목 + 설명 + 스크린샷 (가장 많이 사용)
- `section-divider`: 중앙 큰 텍스트
- `contact`: 연락처 정보 레이아웃
