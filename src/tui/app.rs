/// TUI 应用
///
/// 交互式菜单界面
use crate::core::{QuantixError, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TuiMenuAction {
    DataSync,
    StrategyRun,
    Backtest,
    TaskManagement,
    TechnicalAnalysis,
    DataExport,
    Exit,
}

#[derive(Debug, Clone, Copy)]
pub struct TuiMenuItem {
    pub action: TuiMenuAction,
    pub label: &'static str,
    pub description: &'static str,
}

const MENU_ITEMS: &[TuiMenuItem] = &[
    TuiMenuItem {
        action: TuiMenuAction::DataSync,
        label: "数据同步",
        description: "查询、导出K线数据，管理数据源配置",
    },
    TuiMenuItem {
        action: TuiMenuAction::StrategyRun,
        label: "策略运行",
        description: "查看、创建和管理交易策略",
    },
    TuiMenuItem {
        action: TuiMenuAction::Backtest,
        label: "回测分析",
        description: "用历史数据验证策略表现",
    },
    TuiMenuItem {
        action: TuiMenuAction::TaskManagement,
        label: "任务管理",
        description: "查看、启动预置定时任务",
    },
    TuiMenuItem {
        action: TuiMenuAction::TechnicalAnalysis,
        label: "技术分析",
        description: "计算MA/RSI/MACD等技术指标",
    },
    TuiMenuItem {
        action: TuiMenuAction::DataExport,
        label: "数据导出",
        description: "导出K线数据为CSV或Parquet格式",
    },
    TuiMenuItem {
        action: TuiMenuAction::Exit,
        label: "退出",
        description: "返回终端",
    },
];

/// 返回 TUI 菜单项的静态切片（包含 label + action）；长度与顺序与 MENU_ITEMS 常量一致。
pub fn menu_items() -> &'static [TuiMenuItem] {
    MENU_ITEMS
}

#[derive(Debug, Clone, Default)]
pub struct TuiMenuState {
    selected: usize,
}

impl TuiMenuState {
    /// 把选中位置下移一项，到末尾后回绕到 0。
    pub fn next(&mut self) {
        self.selected = (self.selected + 1) % menu_items().len();
    }

    /// 把选中位置上移一项，到 0 后回绕到末尾。
    pub fn previous(&mut self) {
        self.selected = self
            .selected
            .checked_sub(1)
            .unwrap_or_else(|| menu_items().len() - 1);
    }

    /// 返回当前选中项的 TuiMenuAction，供 dispatcher 据此路由到子命令。
    pub fn selected_action(&self) -> TuiMenuAction {
        menu_items()[self.selected].action
    }
}

/// 运行交互式菜单
#[cfg(feature = "tui")]
pub fn run_menu() -> Result<TuiMenuAction> {
    use crossterm::{
        execute,
        terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
    };
    use ratatui::{Terminal, backend::CrosstermBackend};
    use std::io;

    enable_raw_mode().map_err(to_tui_error)?;
    let mut stdout = io::stdout();
    if let Err(error) = execute!(stdout, EnterAlternateScreen) {
        if let Err(err) = disable_raw_mode() {
            tracing::warn!("TUI 退出 raw_mode 失败: {}", err);
        }
        return Err(to_tui_error(error));
    }

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = match Terminal::new(backend) {
        Ok(terminal) => terminal,
        Err(error) => {
            if let Err(err) = disable_raw_mode() {
                tracing::warn!("TUI 退出 raw_mode 失败: {}", err);
            }
            return Err(to_tui_error(error));
        }
    };

    let result = run_event_loop(&mut terminal);
    let raw_mode_result = disable_raw_mode().map_err(to_tui_error);
    let screen_result =
        execute!(terminal.backend_mut(), LeaveAlternateScreen).map_err(to_tui_error);
    let cursor_result = terminal.show_cursor().map_err(to_tui_error);

    match (
        result,
        raw_mode_result.and(screen_result).and(cursor_result),
    ) {
        (Ok(action), Ok(())) => Ok(action),
        (Err(error), _) => Err(error),
        (_, Err(error)) => Err(error),
    }
}

#[cfg(not(feature = "tui"))]
pub fn run_menu() -> Result<TuiMenuAction> {
    Err(QuantixError::Other(
        "TUI 功能未启用，请使用 --features tui 构建后再运行 menu --tui".to_string(),
    ))
}

#[cfg(feature = "tui")]
fn run_event_loop<B>(terminal: &mut ratatui::Terminal<B>) -> Result<TuiMenuAction>
where
    B: ratatui::backend::Backend,
{
    use crossterm::event::{self, Event, KeyCode};

    let mut state = TuiMenuState::default();
    loop {
        terminal
            .draw(|frame| render_menu(frame, &state))
            .map_err(to_tui_error)?;

        if let Event::Key(key) = event::read().map_err(to_tui_error)? {
            match key.code {
                KeyCode::Down | KeyCode::Char('j') => state.next(),
                KeyCode::Up | KeyCode::Char('k') => state.previous(),
                KeyCode::Enter => return Ok(state.selected_action()),
                KeyCode::Esc | KeyCode::Char('q') => return Ok(TuiMenuAction::Exit),
                _ => {}
            }
        }
    }
}

#[cfg(feature = "tui")]
fn render_menu(frame: &mut ratatui::Frame<'_>, state: &TuiMenuState) {
    use ratatui::{
        layout::{Alignment, Constraint, Direction, Layout},
        style::{Color, Modifier, Style},
        text::{Line, Span},
        widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Min(10),
            Constraint::Length(3),
        ])
        .split(frame.size());

    let title = Paragraph::new("Quantix CLI")
        .alignment(Alignment::Center)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL).title("TUI Menu"));
    frame.render_widget(title, chunks[0]);

    let items = menu_items()
        .iter()
        .map(|item| {
            ListItem::new(Line::from(vec![
                Span::styled(
                    format!("{:<12}", item.label),
                    Style::default().add_modifier(Modifier::BOLD),
                ),
                Span::raw(item.description),
            ]))
        })
        .collect::<Vec<_>>();

    let mut list_state = ListState::default();
    list_state.select(Some(state.selected));
    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).title("主菜单"))
        .highlight_style(
            Style::default()
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");
    frame.render_stateful_widget(list, chunks[1], &mut list_state);

    let help = Paragraph::new("↑/↓ 或 j/k 移动  Enter 选择  q/Esc 退出")
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(help, chunks[2]);
}

#[cfg(feature = "tui")]
fn to_tui_error(error: impl std::fmt::Display) -> QuantixError {
    QuantixError::Other(format!("TUI 错误: {}", error))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_model_exposes_simple_menu_actions() {
        let items = menu_items();

        assert_eq!(items.len(), 7);
        assert_eq!(items[0].action, TuiMenuAction::DataSync);
        assert_eq!(items[0].label, "数据同步");
        assert_eq!(items[1].action, TuiMenuAction::StrategyRun);
        assert_eq!(items[6].action, TuiMenuAction::Exit);
    }

    #[test]
    fn menu_state_wraps_selection() {
        let mut state = TuiMenuState::default();

        state.previous();
        assert_eq!(state.selected_action(), TuiMenuAction::Exit);

        state.next();
        assert_eq!(state.selected_action(), TuiMenuAction::DataSync);
    }

    #[cfg(feature = "tui")]
    #[test]
    fn render_menu_draws_title_and_items() {
        use ratatui::{Terminal, backend::TestBackend};

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).unwrap();
        let state = TuiMenuState::default();

        terminal.draw(|frame| render_menu(frame, &state)).unwrap();
        let screen = terminal
            .backend()
            .buffer()
            .content
            .iter()
            .map(|cell| cell.symbol())
            .collect::<Vec<_>>()
            .join("");
        let compact_screen = screen
            .chars()
            .filter(|character| !character.is_whitespace())
            .collect::<String>();

        assert!(screen.contains("Quantix CLI"));
        assert!(compact_screen.contains("数据同步"));
        assert!(compact_screen.contains("策略运行"));
        assert!(compact_screen.contains("↑/↓"));
    }
}
