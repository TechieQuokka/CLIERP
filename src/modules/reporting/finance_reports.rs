use chrono::Utc;
use std::collections::HashMap;
use crate::core::result::CLIERPResult;
use super::engine::*;

pub struct FinanceReportsGenerator;

impl ReportGenerator for FinanceReportsGenerator {
    fn generate_report(&self, config: ReportConfig) -> Result<ReportResult> {
        match config.title.as_str() {
            "income_statement" => self.generate_income_statement(config),
            "balance_sheet" => self.generate_balance_sheet(config),
            "cash_flow" => self.generate_cash_flow_statement(config),
            "budget_vs_actual" => self.generate_budget_vs_actual_report(config),
            "financial_analytics" => self.generate_financial_analytics(config),
            _ => Err(crate::core::error::AppError::NotFound(
                format!("Finance report '{}' not found", config.title)
            )),
        }
    }

    fn get_available_filters(&self) -> Vec<FilterDefinition> {
        vec![
            FilterDefinition {
                name: "period".to_string(),
                label: "Period".to_string(),
                filter_type: FilterType::Select,
                required: true,
                default_value: Some("monthly".to_string()),
                options: Some(vec![
                    FilterOption { value: "monthly".to_string(), label: "Monthly".to_string() },
                    FilterOption { value: "quarterly".to_string(), label: "Quarterly".to_string() },
                    FilterOption { value: "yearly".to_string(), label: "Yearly".to_string() },
                ]),
            },
            FilterDefinition {
                name: "account_type".to_string(),
                label: "Account Type".to_string(),
                filter_type: FilterType::MultiSelect,
                required: false,
                default_value: None,
                options: Some(vec![
                    FilterOption { value: "asset".to_string(), label: "Assets".to_string() },
                    FilterOption { value: "liability".to_string(), label: "Liabilities".to_string() },
                    FilterOption { value: "equity".to_string(), label: "Equity".to_string() },
                    FilterOption { value: "revenue".to_string(), label: "Revenue".to_string() },
                    FilterOption { value: "expense".to_string(), label: "Expenses".to_string() },
                ]),
            },
            FilterDefinition {
                name: "comparison".to_string(),
                label: "Compare with Previous Period".to_string(),
                filter_type: FilterType::Boolean,
                required: false,
                default_value: Some("true".to_string()),
                options: None,
            },
        ]
    }

    fn get_report_info(&self) -> ReportInfo {
        ReportInfo {
            id: "finance_reports".to_string(),
            name: "Financial Reports".to_string(),
            description: "Comprehensive financial reporting including P&L, Balance Sheet, and Cash Flow statements".to_string(),
            category: "Finance".to_string(),
            supported_formats: vec![
                ReportFormat::Json,
                ReportFormat::Csv,
                ReportFormat::Html,
                ReportFormat::Text,
            ],
        }
    }
}

impl FinanceReportsGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_income_statement(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Revenue".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Account".to_string(),
                        "Current Period".to_string(),
                        "Previous Period".to_string(),
                        "Change".to_string(),
                        "% Change".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Sales Revenue".to_string(),
                            "₩850,000,000".to_string(),
                            "₩780,000,000".to_string(),
                            "₩70,000,000".to_string(),
                            "+9.0%".to_string(),
                        ],
                        vec![
                            "Service Revenue".to_string(),
                            "₩245,000,000".to_string(),
                            "₩220,000,000".to_string(),
                            "₩25,000,000".to_string(),
                            "+11.4%".to_string(),
                        ],
                        vec![
                            "Other Income".to_string(),
                            "₩15,000,000".to_string(),
                            "₩12,000,000".to_string(),
                            "₩3,000,000".to_string(),
                            "+25.0%".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total Revenue".to_string(),
                        "₩1,110,000,000".to_string(),
                        "₩1,012,000,000".to_string(),
                        "₩98,000,000".to_string(),
                        "+9.7%".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Expenses".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Account".to_string(),
                        "Current Period".to_string(),
                        "Previous Period".to_string(),
                        "Change".to_string(),
                        "% Change".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Cost of Goods Sold".to_string(),
                            "₩455,000,000".to_string(),
                            "₩420,000,000".to_string(),
                            "₩35,000,000".to_string(),
                            "+8.3%".to_string(),
                        ],
                        vec![
                            "Salaries & Benefits".to_string(),
                            "₩320,000,000".to_string(),
                            "₩305,000,000".to_string(),
                            "₩15,000,000".to_string(),
                            "+4.9%".to_string(),
                        ],
                        vec![
                            "Operating Expenses".to_string(),
                            "₩180,000,000".to_string(),
                            "₩175,000,000".to_string(),
                            "₩5,000,000".to_string(),
                            "+2.9%".to_string(),
                        ],
                        vec![
                            "Depreciation".to_string(),
                            "₩25,000,000".to_string(),
                            "₩25,000,000".to_string(),
                            "₩0".to_string(),
                            "0.0%".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total Expenses".to_string(),
                        "₩980,000,000".to_string(),
                        "₩925,000,000".to_string(),
                        "₩55,000,000".to_string(),
                        "+5.9%".to_string(),
                    ]),
                }),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_revenue".to_string(), MetricValue::Currency(1110000000));
        key_metrics.insert("total_expenses".to_string(), MetricValue::Currency(980000000));
        key_metrics.insert("net_income".to_string(), MetricValue::Currency(130000000));
        key_metrics.insert("gross_margin".to_string(), MetricValue::Percentage(59.0));
        key_metrics.insert("net_margin".to_string(), MetricValue::Percentage(11.7));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Revenue increased by 9.7% compared to previous period".to_string(),
                "Gross margin improved due to better cost management".to_string(),
                "Operating efficiency increased with controlled expense growth".to_string(),
                "Service revenue showed strongest growth at 11.4%".to_string(),
            ],
            recommendations: vec![
                "Continue focus on high-margin service offerings".to_string(),
                "Optimize cost of goods sold through supplier negotiations".to_string(),
                "Monitor salary growth relative to revenue growth".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 245,
            processing_time_ms: 180,
            filters_applied: vec!["current_month".to_string(), "comparison_enabled".to_string()],
            data_sources: vec!["accounts".to_string(), "transactions".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_balance_sheet(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Assets".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Account".to_string(),
                        "Current".to_string(),
                        "Previous".to_string(),
                        "Change".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Cash and Cash Equivalents".to_string(),
                            "₩450,000,000".to_string(),
                            "₩380,000,000".to_string(),
                            "₩70,000,000".to_string(),
                        ],
                        vec![
                            "Accounts Receivable".to_string(),
                            "₩320,000,000".to_string(),
                            "₩295,000,000".to_string(),
                            "₩25,000,000".to_string(),
                        ],
                        vec![
                            "Inventory".to_string(),
                            "₩180,000,000".to_string(),
                            "₩165,000,000".to_string(),
                            "₩15,000,000".to_string(),
                        ],
                        vec![
                            "Property, Plant & Equipment".to_string(),
                            "₩850,000,000".to_string(),
                            "₩875,000,000".to_string(),
                            "-₩25,000,000".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total Assets".to_string(),
                        "₩1,800,000,000".to_string(),
                        "₩1,715,000,000".to_string(),
                        "₩85,000,000".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Liabilities & Equity".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Account".to_string(),
                        "Current".to_string(),
                        "Previous".to_string(),
                        "Change".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Accounts Payable".to_string(),
                            "₩185,000,000".to_string(),
                            "₩175,000,000".to_string(),
                            "₩10,000,000".to_string(),
                        ],
                        vec![
                            "Short-term Debt".to_string(),
                            "₩120,000,000".to_string(),
                            "₩150,000,000".to_string(),
                            "-₩30,000,000".to_string(),
                        ],
                        vec![
                            "Long-term Debt".to_string(),
                            "₩450,000,000".to_string(),
                            "₩480,000,000".to_string(),
                            "-₩30,000,000".to_string(),
                        ],
                        vec![
                            "Retained Earnings".to_string(),
                            "₩780,000,000".to_string(),
                            "₩650,000,000".to_string(),
                            "₩130,000,000".to_string(),
                        ],
                        vec![
                            "Share Capital".to_string(),
                            "₩265,000,000".to_string(),
                            "₩260,000,000".to_string(),
                            "₩5,000,000".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total Liabilities & Equity".to_string(),
                        "₩1,800,000,000".to_string(),
                        "₩1,715,000,000".to_string(),
                        "₩85,000,000".to_string(),
                    ]),
                }),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_assets".to_string(), MetricValue::Currency(1800000000));
        key_metrics.insert("total_liabilities".to_string(), MetricValue::Currency(755000000));
        key_metrics.insert("total_equity".to_string(), MetricValue::Currency(1045000000));
        key_metrics.insert("debt_to_equity_ratio".to_string(), MetricValue::Number(0.72));
        key_metrics.insert("current_ratio".to_string(), MetricValue::Number(3.1));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Asset base grew by ₩85 million driven by cash and receivables".to_string(),
                "Debt levels decreased by ₩60 million improving financial position".to_string(),
                "Current ratio of 3.1 indicates strong liquidity".to_string(),
                "Retained earnings increased by ₩130 million from profitable operations".to_string(),
            ],
            recommendations: vec![
                "Consider investing excess cash in growth opportunities".to_string(),
                "Maintain current debt reduction strategy".to_string(),
                "Optimize accounts receivable collection processes".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 58,
            processing_time_ms: 95,
            filters_applied: vec!["balance_sheet_accounts".to_string()],
            data_sources: vec!["accounts".to_string(), "transactions".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_cash_flow_statement(&self, config: ReportConfig) -> Result<ReportResult> {
        let table_data = TableData {
            headers: vec![
                "Cash Flow Category".to_string(),
                "Current Month".to_string(),
                "Previous Month".to_string(),
                "YTD".to_string(),
            ],
            rows: vec![
                vec![
                    "Operating Activities".to_string(),
                    "₩145,000,000".to_string(),
                    "₩125,000,000".to_string(),
                    "₩890,000,000".to_string(),
                ],
                vec![
                    "  Net Income".to_string(),
                    "₩130,000,000".to_string(),
                    "₩115,000,000".to_string(),
                    "₩780,000,000".to_string(),
                ],
                vec![
                    "  Depreciation".to_string(),
                    "₩25,000,000".to_string(),
                    "₩25,000,000".to_string(),
                    "₩300,000,000".to_string(),
                ],
                vec![
                    "  Working Capital Changes".to_string(),
                    "-₩10,000,000".to_string(),
                    "-₩15,000,000".to_string(),
                    "-₩190,000,000".to_string(),
                ],
                vec![
                    "Investing Activities".to_string(),
                    "-₩45,000,000".to_string(),
                    "-₩35,000,000".to_string(),
                    "-₩320,000,000".to_string(),
                ],
                vec![
                    "  Capital Expenditures".to_string(),
                    "-₩50,000,000".to_string(),
                    "-₩40,000,000".to_string(),
                    "-₩350,000,000".to_string(),
                ],
                vec![
                    "  Asset Disposals".to_string(),
                    "₩5,000,000".to_string(),
                    "₩5,000,000".to_string(),
                    "₩30,000,000".to_string(),
                ],
                vec![
                    "Financing Activities".to_string(),
                    "-₩30,000,000".to_string(),
                    "-₩25,000,000".to_string(),
                    "-₩200,000,000".to_string(),
                ],
                vec![
                    "  Debt Payments".to_string(),
                    "-₩25,000,000".to_string(),
                    "-₩20,000,000".to_string(),
                    "-₩180,000,000".to_string(),
                ],
                vec![
                    "  Dividend Payments".to_string(),
                    "-₩5,000,000".to_string(),
                    "-₩5,000,000".to_string(),
                    "-₩20,000,000".to_string(),
                ],
            ],
            totals: Some(vec![
                "Net Cash Flow".to_string(),
                "₩70,000,000".to_string(),
                "₩65,000,000".to_string(),
                "₩370,000,000".to_string(),
            ]),
        };

        let mut key_metrics = HashMap::new();
        key_metrics.insert("operating_cash_flow".to_string(), MetricValue::Currency(145000000));
        key_metrics.insert("free_cash_flow".to_string(), MetricValue::Currency(95000000));
        key_metrics.insert("cash_conversion_cycle".to_string(), MetricValue::Number(45.2));
        key_metrics.insert("operating_margin".to_string(), MetricValue::Percentage(13.1));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Operating cash flow increased by 16% month-over-month".to_string(),
                "Free cash flow of ₩95M indicates strong cash generation".to_string(),
                "Capital expenditures focused on growth initiatives".to_string(),
                "Debt reduction continues as planned".to_string(),
            ],
            recommendations: vec![
                "Continue focus on operating cash flow optimization".to_string(),
                "Evaluate ROI of recent capital investments".to_string(),
                "Consider increasing dividend based on strong cash position".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 156,
            processing_time_ms: 145,
            filters_applied: vec!["cash_flow_accounts".to_string()],
            data_sources: vec!["transactions".to_string(), "accounts".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Table(table_data),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_budget_vs_actual_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Revenue Analysis".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Revenue Stream".to_string(),
                        "Budget".to_string(),
                        "Actual".to_string(),
                        "Variance".to_string(),
                        "% Variance".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Product Sales".to_string(),
                            "₩800,000,000".to_string(),
                            "₩850,000,000".to_string(),
                            "₩50,000,000".to_string(),
                            "+6.3%".to_string(),
                        ],
                        vec![
                            "Services".to_string(),
                            "₩200,000,000".to_string(),
                            "₩245,000,000".to_string(),
                            "₩45,000,000".to_string(),
                            "+22.5%".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total Revenue".to_string(),
                        "₩1,000,000,000".to_string(),
                        "₩1,095,000,000".to_string(),
                        "₩95,000,000".to_string(),
                        "+9.5%".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Expense Analysis".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Expense Category".to_string(),
                        "Budget".to_string(),
                        "Actual".to_string(),
                        "Variance".to_string(),
                        "% Variance".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Personnel".to_string(),
                            "₩300,000,000".to_string(),
                            "₩320,000,000".to_string(),
                            "₩20,000,000".to_string(),
                            "+6.7%".to_string(),
                        ],
                        vec![
                            "Operations".to_string(),
                            "₩450,000,000".to_string(),
                            "₩435,000,000".to_string(),
                            "-₩15,000,000".to_string(),
                            "-3.3%".to_string(),
                        ],
                        vec![
                            "Marketing".to_string(),
                            "₩80,000,000".to_string(),
                            "₩85,000,000".to_string(),
                            "₩5,000,000".to_string(),
                            "+6.3%".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total Expenses".to_string(),
                        "₩830,000,000".to_string(),
                        "₩840,000,000".to_string(),
                        "₩10,000,000".to_string(),
                        "+1.2%".to_string(),
                    ]),
                }),
            ),
            ReportSection {
                title: "Budget Performance Chart".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_bar_chart(
                    vec![
                        "Q1".to_string(),
                        "Q2".to_string(),
                        "Q3".to_string(),
                        "Q4".to_string(),
                    ],
                    vec![105.2, 98.7, 109.5, 102.1],
                    "Budget Achievement %",
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("revenue_variance".to_string(), MetricValue::Currency(95000000));
        key_metrics.insert("expense_variance".to_string(), MetricValue::Currency(10000000));
        key_metrics.insert("net_variance".to_string(), MetricValue::Currency(85000000));
        key_metrics.insert("budget_achievement".to_string(), MetricValue::Percentage(105.2));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Revenue exceeded budget by 9.5% driven by strong service growth".to_string(),
                "Expenses remained well-controlled with only 1.2% over budget".to_string(),
                "Service revenue performed exceptionally with 22.5% above budget".to_string(),
                "Operational efficiency improved with cost savings in operations".to_string(),
            ],
            recommendations: vec![
                "Revise service revenue targets upward for next period".to_string(),
                "Investigate personnel cost increases for sustainability".to_string(),
                "Consider reallocating marketing budget to high-performing channels".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 89,
            processing_time_ms: 125,
            filters_applied: vec!["current_period".to_string(), "budget_comparison".to_string()],
            data_sources: vec!["budget".to_string(), "actuals".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_financial_analytics(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Financial Ratios".to_string(),
                section_type: SectionType::Analysis,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Ratio".to_string(),
                        "Current".to_string(),
                        "Previous".to_string(),
                        "Industry Avg".to_string(),
                        "Status".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Current Ratio".to_string(),
                            "3.1".to_string(),
                            "2.8".to_string(),
                            "2.5".to_string(),
                            "Above Average".to_string(),
                        ],
                        vec![
                            "Debt-to-Equity".to_string(),
                            "0.72".to_string(),
                            "0.84".to_string(),
                            "0.65".to_string(),
                            "Improving".to_string(),
                        ],
                        vec![
                            "ROE".to_string(),
                            "12.4%".to_string(),
                            "11.2%".to_string(),
                            "10.5%".to_string(),
                            "Above Average".to_string(),
                        ],
                        vec![
                            "ROA".to_string(),
                            "7.2%".to_string(),
                            "6.7%".to_string(),
                            "6.8%".to_string(),
                            "Above Average".to_string(),
                        ],
                    ],
                    totals: None,
                }),
            },
            ReportSection {
                title: "Trend Analysis".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_line_chart(
                    vec![
                        "Jan".to_string(),
                        "Feb".to_string(),
                        "Mar".to_string(),
                        "Apr".to_string(),
                        "May".to_string(),
                        "Jun".to_string(),
                    ],
                    vec![
                        Dataset {
                            label: "Revenue Growth %".to_string(),
                            data: vec![8.5, 9.2, 7.8, 11.3, 9.7, 10.1],
                            color: Some("#10B981".to_string()),
                        },
                        Dataset {
                            label: "Profit Margin %".to_string(),
                            data: vec![11.2, 11.8, 10.9, 12.1, 11.7, 12.4],
                            color: Some("#3B82F6".to_string()),
                        },
                    ],
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("current_ratio".to_string(), MetricValue::Number(3.1));
        key_metrics.insert("roe".to_string(), MetricValue::Percentage(12.4));
        key_metrics.insert("roa".to_string(), MetricValue::Percentage(7.2));
        key_metrics.insert("debt_to_equity".to_string(), MetricValue::Number(0.72));
        key_metrics.insert("financial_health_score".to_string(), MetricValue::Number(8.7));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Financial health improved across all key metrics".to_string(),
                "Liquidity position strengthened with current ratio of 3.1".to_string(),
                "Profitability trends show consistent improvement".to_string(),
                "Debt management strategy yielding positive results".to_string(),
            ],
            recommendations: vec![
                "Maintain current financial discipline".to_string(),
                "Consider strategic investments to deploy excess liquidity".to_string(),
                "Continue debt reduction to reach industry benchmark".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 24,
            processing_time_ms: 95,
            filters_applied: vec!["financial_ratios".to_string()],
            data_sources: vec!["balance_sheet".to_string(), "income_statement".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }
}

impl Default for FinanceReportsGenerator {
    fn default() -> Self {
        Self::new()
    }
}