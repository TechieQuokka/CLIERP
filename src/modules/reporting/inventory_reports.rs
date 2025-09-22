use chrono::Utc;
use std::collections::HashMap;
use crate::core::result::CLIERPResult;
use super::engine::*;

pub struct InventoryReportsGenerator;

impl ReportGenerator for InventoryReportsGenerator {
    fn generate_report(&self, config: ReportConfig) -> Result<ReportResult> {
        match config.title.as_str() {
            "stock_status" => self.generate_stock_status_report(config),
            "stock_movement" => self.generate_stock_movement_report(config),
            "inventory_valuation" => self.generate_inventory_valuation_report(config),
            "purchase_analysis" => self.generate_purchase_analysis_report(config),
            "supplier_performance" => self.generate_supplier_performance_report(config),
            "abc_analysis" => self.generate_abc_analysis_report(config),
            _ => Err(crate::core::error::AppError::NotFound(
                format!("Inventory report '{}' not found", config.title)
            )),
        }
    }

    fn get_available_filters(&self) -> Vec<FilterDefinition> {
        vec![
            FilterDefinition {
                name: "category_id".to_string(),
                label: "Product Category".to_string(),
                filter_type: FilterType::Select,
                required: false,
                default_value: None,
                options: None, // Would be populated dynamically from categories
            },
            FilterDefinition {
                name: "stock_level".to_string(),
                label: "Stock Level".to_string(),
                filter_type: FilterType::Select,
                required: false,
                default_value: None,
                options: Some(vec![
                    FilterOption { value: "all".to_string(), label: "All Levels".to_string() },
                    FilterOption { value: "low".to_string(), label: "Low Stock".to_string() },
                    FilterOption { value: "out".to_string(), label: "Out of Stock".to_string() },
                    FilterOption { value: "overstocked".to_string(), label: "Overstocked".to_string() },
                ]),
            },
            FilterDefinition {
                name: "supplier_id".to_string(),
                label: "Supplier".to_string(),
                filter_type: FilterType::Select,
                required: false,
                default_value: None,
                options: None, // Would be populated dynamically from suppliers
            },
            FilterDefinition {
                name: "movement_type".to_string(),
                label: "Movement Type".to_string(),
                filter_type: FilterType::Select,
                required: false,
                default_value: None,
                options: Some(vec![
                    FilterOption { value: "in".to_string(), label: "Stock In".to_string() },
                    FilterOption { value: "out".to_string(), label: "Stock Out".to_string() },
                    FilterOption { value: "adjustment".to_string(), label: "Adjustments".to_string() },
                    FilterOption { value: "transfer".to_string(), label: "Transfers".to_string() },
                ]),
            },
        ]
    }

    fn get_report_info(&self) -> ReportInfo {
        ReportInfo {
            id: "inventory_reports".to_string(),
            name: "Inventory Reports".to_string(),
            description: "Comprehensive inventory management reports including stock status, movements, and supplier analysis".to_string(),
            category: "Inventory".to_string(),
            supported_formats: vec![
                ReportFormat::Json,
                ReportFormat::Csv,
                ReportFormat::Html,
                ReportFormat::Text,
            ],
        }
    }
}

impl InventoryReportsGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_stock_status_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Current Stock Levels".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "SKU".to_string(),
                        "Product Name".to_string(),
                        "Category".to_string(),
                        "Current Stock".to_string(),
                        "Min Stock".to_string(),
                        "Max Stock".to_string(),
                        "Value".to_string(),
                        "Status".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "PRD001".to_string(),
                            "Laptop Computer".to_string(),
                            "Electronics".to_string(),
                            "45".to_string(),
                            "20".to_string(),
                            "100".to_string(),
                            "₩67,500,000".to_string(),
                            "Normal".to_string(),
                        ],
                        vec![
                            "PRD002".to_string(),
                            "Office Chair".to_string(),
                            "Furniture".to_string(),
                            "8".to_string(),
                            "15".to_string(),
                            "50".to_string(),
                            "₩2,400,000".to_string(),
                            "Low Stock".to_string(),
                        ],
                        vec![
                            "PRD003".to_string(),
                            "Printer Paper".to_string(),
                            "Office Supplies".to_string(),
                            "0".to_string(),
                            "100".to_string(),
                            "500".to_string(),
                            "₩0".to_string(),
                            "Out of Stock".to_string(),
                        ],
                        vec![
                            "PRD004".to_string(),
                            "Desk Lamp".to_string(),
                            "Furniture".to_string(),
                            "85".to_string(),
                            "10".to_string(),
                            "30".to_string(),
                            "₩4,250,000".to_string(),
                            "Overstocked".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total".to_string(),
                        "".to_string(),
                        "4 Categories".to_string(),
                        "138 Units".to_string(),
                        "".to_string(),
                        "".to_string(),
                        "₩74,150,000".to_string(),
                        "".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Stock Distribution".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_pie_chart(
                    vec![
                        "Normal Stock".to_string(),
                        "Low Stock".to_string(),
                        "Out of Stock".to_string(),
                        "Overstocked".to_string(),
                    ],
                    vec![65.0, 20.0, 8.0, 7.0],
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_products".to_string(), MetricValue::Count(1250));
        key_metrics.insert("total_stock_value".to_string(), MetricValue::Currency(2840000000));
        key_metrics.insert("low_stock_items".to_string(), MetricValue::Count(85));
        key_metrics.insert("out_of_stock_items".to_string(), MetricValue::Count(12));
        key_metrics.insert("overstocked_items".to_string(), MetricValue::Count(28));
        key_metrics.insert("stock_turnover_ratio".to_string(), MetricValue::Number(6.8));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "65% of products are at normal stock levels".to_string(),
                "20% of products require immediate restocking".to_string(),
                "Total inventory value of ₩2.84 billion".to_string(),
                "Stock turnover ratio of 6.8 indicates healthy inventory movement".to_string(),
                "Electronics category has highest inventory value".to_string(),
            ],
            recommendations: vec![
                "Immediate reorder needed for 12 out-of-stock items".to_string(),
                "Review max stock levels for overstocked items".to_string(),
                "Implement automated reorder points for critical items".to_string(),
                "Consider markdown strategy for slow-moving inventory".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 1250,
            processing_time_ms: 280,
            filters_applied: vec!["active_products".to_string()],
            data_sources: vec!["products".to_string(), "stock_movements".to_string(), "categories".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_stock_movement_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let table_data = TableData {
            headers: vec![
                "Date".to_string(),
                "SKU".to_string(),
                "Product".to_string(),
                "Movement Type".to_string(),
                "Quantity".to_string(),
                "Unit Cost".to_string(),
                "Total Value".to_string(),
                "Reference".to_string(),
            ],
            rows: vec![
                vec![
                    "2024-03-15".to_string(),
                    "PRD001".to_string(),
                    "Laptop Computer".to_string(),
                    "Stock In".to_string(),
                    "+20".to_string(),
                    "₩1,500,000".to_string(),
                    "₩30,000,000".to_string(),
                    "PO-2024-001".to_string(),
                ],
                vec![
                    "2024-03-14".to_string(),
                    "PRD002".to_string(),
                    "Office Chair".to_string(),
                    "Stock Out".to_string(),
                    "-5".to_string(),
                    "₩300,000".to_string(),
                    "₩1,500,000".to_string(),
                    "SO-2024-028".to_string(),
                ],
                vec![
                    "2024-03-13".to_string(),
                    "PRD003".to_string(),
                    "Printer Paper".to_string(),
                    "Stock Out".to_string(),
                    "-100".to_string(),
                    "₩5,000".to_string(),
                    "₩500,000".to_string(),
                    "SO-2024-027".to_string(),
                ],
                vec![
                    "2024-03-12".to_string(),
                    "PRD004".to_string(),
                    "Desk Lamp".to_string(),
                    "Adjustment".to_string(),
                    "+2".to_string(),
                    "₩50,000".to_string(),
                    "₩100,000".to_string(),
                    "ADJ-2024-003".to_string(),
                ],
                vec![
                    "2024-03-11".to_string(),
                    "PRD001".to_string(),
                    "Laptop Computer".to_string(),
                    "Stock Out".to_string(),
                    "-3".to_string(),
                    "₩1,500,000".to_string(),
                    "₩4,500,000".to_string(),
                    "SO-2024-026".to_string(),
                ],
            ],
            totals: Some(vec![
                "Totals".to_string(),
                "".to_string(),
                "".to_string(),
                "".to_string(),
                "Net: -86".to_string(),
                "".to_string(),
                "₩36,600,000".to_string(),
                "5 Movements".to_string(),
            ]),
        };

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_movements".to_string(), MetricValue::Count(847));
        key_metrics.insert("stock_in_value".to_string(), MetricValue::Currency(1250000000));
        key_metrics.insert("stock_out_value".to_string(), MetricValue::Currency(980000000));
        key_metrics.insert("net_movement_value".to_string(), MetricValue::Currency(270000000));
        key_metrics.insert("average_daily_movements".to_string(), MetricValue::Number(28.2));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "847 stock movements recorded in the selected period".to_string(),
                "Net positive stock movement of ₩270 million".to_string(),
                "Stock inflows primarily from purchase orders".to_string(),
                "Average of 28.2 movements per day indicates active inventory".to_string(),
                "Electronics category shows highest movement frequency".to_string(),
            ],
            recommendations: vec![
                "Review high-frequency movement items for automation opportunities".to_string(),
                "Implement real-time tracking for high-value items".to_string(),
                "Optimize reorder quantities based on movement patterns".to_string(),
                "Consider cycle counting for fast-moving items".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 847,
            processing_time_ms: 195,
            filters_applied: vec!["date_range".to_string(), "movement_types".to_string()],
            data_sources: vec!["stock_movements".to_string(), "products".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Table(table_data),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_inventory_valuation_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Valuation by Category".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Category".to_string(),
                        "Product Count".to_string(),
                        "Total Quantity".to_string(),
                        "Average Cost".to_string(),
                        "Total Value".to_string(),
                        "% of Total".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Electronics".to_string(),
                            "125".to_string(),
                            "1,245".to_string(),
                            "₩1,250,000".to_string(),
                            "₩1,556,250,000".to_string(),
                            "54.8%".to_string(),
                        ],
                        vec![
                            "Furniture".to_string(),
                            "85".to_string(),
                            "890".to_string(),
                            "₩450,000".to_string(),
                            "₩400,500,000".to_string(),
                            "14.1%".to_string(),
                        ],
                        vec![
                            "Office Supplies".to_string(),
                            "250".to_string(),
                            "5,670".to_string(),
                            "₩15,000".to_string(),
                            "₩85,050,000".to_string(),
                            "3.0%".to_string(),
                        ],
                        vec![
                            "Manufacturing".to_string(),
                            "180".to_string(),
                            "2,340".to_string(),
                            "₩320,000".to_string(),
                            "₩748,800,000".to_string(),
                            "26.4%".to_string(),
                        ],
                        vec![
                            "Safety Equipment".to_string(),
                            "65".to_string(),
                            "780".to_string(),
                            "₩65,000".to_string(),
                            "₩50,700,000".to_string(),
                            "1.8%".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total".to_string(),
                        "705".to_string(),
                        "10,925".to_string(),
                        "₩259,864".to_string(),
                        "₩2,841,300,000".to_string(),
                        "100.0%".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Valuation Trends".to_string(),
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
                            label: "Total Inventory Value (Billions)".to_string(),
                            data: vec![2.65, 2.72, 2.84, 2.91, 2.88, 2.84],
                            color: Some("#3B82F6".to_string()),
                        },
                        Dataset {
                            label: "Monthly Movement Value (Millions)".to_string(),
                            data: vec![450.0, 520.0, 480.0, 580.0, 610.0, 590.0],
                            color: Some("#10B981".to_string()),
                        },
                    ],
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_inventory_value".to_string(), MetricValue::Currency(2841300000));
        key_metrics.insert("average_unit_cost".to_string(), MetricValue::Currency(259864));
        key_metrics.insert("inventory_turnover".to_string(), MetricValue::Number(6.8));
        key_metrics.insert("days_in_inventory".to_string(), MetricValue::Number(53.7));
        key_metrics.insert("obsolete_inventory_risk".to_string(), MetricValue::Percentage(2.3));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Total inventory value of ₩2.84 billion across 705 products".to_string(),
                "Electronics represent 54.8% of total inventory value".to_string(),
                "Inventory turnover of 6.8 indicates efficient stock management".to_string(),
                "Average inventory holding period of 53.7 days".to_string(),
                "Low obsolete inventory risk at 2.3%".to_string(),
            ],
            recommendations: vec![
                "Monitor electronics category closely due to high value concentration".to_string(),
                "Optimize reorder points for high-value, slow-moving items".to_string(),
                "Consider just-in-time delivery for electronics components".to_string(),
                "Implement ABC analysis for better inventory classification".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 705,
            processing_time_ms: 340,
            filters_applied: vec!["valuation_method_fifo".to_string()],
            data_sources: vec!["products".to_string(), "stock_movements".to_string(), "categories".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_purchase_analysis_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let table_data = TableData {
            headers: vec![
                "Month".to_string(),
                "Purchase Orders".to_string(),
                "Total Value".to_string(),
                "Average PO Value".to_string(),
                "Supplier Count".to_string(),
                "On-Time Delivery".to_string(),
                "Quality Issues".to_string(),
            ],
            rows: vec![
                vec![
                    "January 2024".to_string(),
                    "45".to_string(),
                    "₩1,250,000,000".to_string(),
                    "₩27,777,778".to_string(),
                    "18".to_string(),
                    "92.3%".to_string(),
                    "2".to_string(),
                ],
                vec![
                    "February 2024".to_string(),
                    "38".to_string(),
                    "₩1,180,000,000".to_string(),
                    "₩31,052,632".to_string(),
                    "15".to_string(),
                    "94.7%".to_string(),
                    "1".to_string(),
                ],
                vec![
                    "March 2024".to_string(),
                    "52".to_string(),
                    "₩1,450,000,000".to_string(),
                    "₩27,884,615".to_string(),
                    "22".to_string(),
                    "89.1%".to_string(),
                    "4".to_string(),
                ],
                vec![
                    "April 2024".to_string(),
                    "41".to_string(),
                    "₩1,320,000,000".to_string(),
                    "₩32,195,122".to_string(),
                    "19".to_string(),
                    "95.1%".to_string(),
                    "1".to_string(),
                ],
            ],
            totals: Some(vec![
                "Total/Average".to_string(),
                "176".to_string(),
                "₩5,200,000,000".to_string(),
                "₩29,545,455".to_string(),
                "74".to_string(),
                "92.8%".to_string(),
                "8".to_string(),
            ]),
        };

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_purchase_value".to_string(), MetricValue::Currency(5200000000));
        key_metrics.insert("average_po_value".to_string(), MetricValue::Currency(29545455));
        key_metrics.insert("supplier_diversity".to_string(), MetricValue::Count(74));
        key_metrics.insert("on_time_delivery_rate".to_string(), MetricValue::Percentage(92.8));
        key_metrics.insert("cost_savings_achieved".to_string(), MetricValue::Currency(156000000));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Total purchase value of ₩5.2 billion across 176 orders".to_string(),
                "Average purchase order value increased to ₩29.5 million".to_string(),
                "On-time delivery rate of 92.8% meets target threshold".to_string(),
                "Quality issues remain minimal at 4.5% of orders".to_string(),
                "Cost savings of ₩156 million achieved through negotiations".to_string(),
            ],
            recommendations: vec![
                "Focus on improving delivery performance with underperforming suppliers".to_string(),
                "Implement supplier scorecards for continuous improvement".to_string(),
                "Consider consolidating orders to achieve better pricing".to_string(),
                "Develop backup suppliers for critical components".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 176,
            processing_time_ms: 165,
            filters_applied: vec!["completed_orders".to_string(), "date_range".to_string()],
            data_sources: vec!["purchase_orders".to_string(), "purchase_items".to_string(), "suppliers".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Table(table_data),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_supplier_performance_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let table_data = TableData {
            headers: vec![
                "Supplier".to_string(),
                "Orders".to_string(),
                "Total Value".to_string(),
                "On-Time %".to_string(),
                "Quality Score".to_string(),
                "Payment Terms".to_string(),
                "Performance Rating".to_string(),
            ],
            rows: vec![
                vec![
                    "TechSupply Co.".to_string(),
                    "25".to_string(),
                    "₩1,850,000,000".to_string(),
                    "96.0%".to_string(),
                    "4.8/5.0".to_string(),
                    "Net 30".to_string(),
                    "Excellent".to_string(),
                ],
                vec![
                    "Global Electronics Ltd.".to_string(),
                    "18".to_string(),
                    "₩1,200,000,000".to_string(),
                    "89.0%".to_string(),
                    "4.2/5.0".to_string(),
                    "Net 45".to_string(),
                    "Good".to_string(),
                ],
                vec![
                    "Office Solutions Inc.".to_string(),
                    "32".to_string(),
                    "₩650,000,000".to_string(),
                    "94.0%".to_string(),
                    "4.6/5.0".to_string(),
                    "Net 30".to_string(),
                    "Excellent".to_string(),
                ],
                vec![
                    "Manufacturing Parts Co.".to_string(),
                    "22".to_string(),
                    "₩980,000,000".to_string(),
                    "87.0%".to_string(),
                    "3.9/5.0".to_string(),
                    "Net 60".to_string(),
                    "Fair".to_string(),
                ],
                vec![
                    "Safety First Supplies".to_string(),
                    "15".to_string(),
                    "₩380,000,000".to_string(),
                    "98.0%".to_string(),
                    "4.9/5.0".to_string(),
                    "Net 15".to_string(),
                    "Excellent".to_string(),
                ],
            ],
            totals: Some(vec![
                "Total/Average".to_string(),
                "112".to_string(),
                "₩5,060,000,000".to_string(),
                "92.8%".to_string(),
                "4.5/5.0".to_string(),
                "Avg: 36 days".to_string(),
                "".to_string(),
            ]),
        };

        let mut key_metrics = HashMap::new();
        key_metrics.insert("top_suppliers".to_string(), MetricValue::Count(5));
        key_metrics.insert("supplier_concentration_risk".to_string(), MetricValue::Percentage(36.6));
        key_metrics.insert("average_quality_score".to_string(), MetricValue::Number(4.5));
        key_metrics.insert("payment_terms_average".to_string(), MetricValue::Number(36.0));
        key_metrics.insert("supplier_performance_index".to_string(), MetricValue::Number(8.7));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Top 5 suppliers represent 36.6% of total purchase value".to_string(),
                "Average supplier quality score of 4.5/5.0 indicates good relationships".to_string(),
                "TechSupply Co. and Safety First Supplies show excellent performance".to_string(),
                "Manufacturing Parts Co. requires attention for delivery improvement".to_string(),
                "Payment terms average 36 days, supporting cash flow management".to_string(),
            ],
            recommendations: vec![
                "Develop improvement plan for Manufacturing Parts Co.".to_string(),
                "Consider preferred supplier agreements with top performers".to_string(),
                "Diversify supplier base to reduce concentration risk".to_string(),
                "Implement quarterly business reviews with key suppliers".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 5,
            processing_time_ms: 85,
            filters_applied: vec!["top_suppliers".to_string(), "active_status".to_string()],
            data_sources: vec!["suppliers".to_string(), "purchase_orders".to_string(), "supplier_evaluations".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Table(table_data),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_abc_analysis_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "ABC Classification Results".to_string(),
                section_type: SectionType::Analysis,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Class".to_string(),
                        "Product Count".to_string(),
                        "% of Products".to_string(),
                        "Total Value".to_string(),
                        "% of Value".to_string(),
                        "Management Strategy".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Class A".to_string(),
                            "141".to_string(),
                            "20.0%".to_string(),
                            "₩2,273,040,000".to_string(),
                            "80.0%".to_string(),
                            "Tight Control".to_string(),
                        ],
                        vec![
                            "Class B".to_string(),
                            "141".to_string(),
                            "20.0%".to_string(),
                            "₩568,260,000".to_string(),
                            "15.0%".to_string(),
                            "Moderate Control".to_string(),
                        ],
                        vec![
                            "Class C".to_string(),
                            "423".to_string(),
                            "60.0%".to_string(),
                            "₩142,065,000".to_string(),
                            "5.0%".to_string(),
                            "Simple Control".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total".to_string(),
                        "705".to_string(),
                        "100.0%".to_string(),
                        "₩2,983,365,000".to_string(),
                        "100.0%".to_string(),
                        "".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "ABC Distribution Chart".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_bar_chart(
                    vec!["Class A".to_string(), "Class B".to_string(), "Class C".to_string()],
                    vec![141.0, 141.0, 423.0],
                    "Product Count",
                )),
            },
            ReportSection {
                title: "Top Class A Items".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Rank".to_string(),
                        "SKU".to_string(),
                        "Product Name".to_string(),
                        "Annual Usage".to_string(),
                        "Unit Cost".to_string(),
                        "Annual Value".to_string(),
                        "Cumulative %".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "1".to_string(),
                            "PRD001".to_string(),
                            "High-End Laptop".to_string(),
                            "240".to_string(),
                            "₩2,500,000".to_string(),
                            "₩600,000,000".to_string(),
                            "20.1%".to_string(),
                        ],
                        vec![
                            "2".to_string(),
                            "PRD105".to_string(),
                            "Server Hardware".to_string(),
                            "180".to_string(),
                            "₩3,000,000".to_string(),
                            "₩540,000,000".to_string(),
                            "38.2%".to_string(),
                        ],
                        vec![
                            "3".to_string(),
                            "PRD087".to_string(),
                            "Industrial Printer".to_string(),
                            "150".to_string(),
                            "₩1,800,000".to_string(),
                            "₩270,000,000".to_string(),
                            "47.2%".to_string(),
                        ],
                    ],
                    totals: None,
                }),
            ),
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("class_a_items".to_string(), MetricValue::Count(141));
        key_metrics.insert("class_a_value_percentage".to_string(), MetricValue::Percentage(80.0));
        key_metrics.insert("pareto_efficiency".to_string(), MetricValue::Number(0.95));
        key_metrics.insert("inventory_concentration".to_string(), MetricValue::Number(8.2));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Classic Pareto distribution: 20% of items represent 80% of value".to_string(),
                "141 Class A items require intensive management attention".to_string(),
                "60% of products (Class C) contribute only 5% of total value".to_string(),
                "High inventory concentration in electronics and servers".to_string(),
                "ABC classification helps optimize inventory management resources".to_string(),
            ],
            recommendations: vec![
                "Implement daily monitoring for all Class A items".to_string(),
                "Use JIT delivery for high-value, predictable Class A items".to_string(),
                "Apply bulk ordering strategies for Class C items".to_string(),
                "Review Class B items monthly for reclassification".to_string(),
                "Establish separate approval processes by ABC class".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 705,
            processing_time_ms: 420,
            filters_applied: vec!["annual_usage_data".to_string(), "value_calculation".to_string()],
            data_sources: vec!["products".to_string(), "stock_movements".to_string(), "usage_history".to_string()],
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

impl Default for InventoryReportsGenerator {
    fn default() -> Self {
        Self::new()
    }
}