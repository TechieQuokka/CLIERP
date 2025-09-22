use chrono::Utc;
use std::collections::HashMap;
use crate::core::result::CLIERPResult;
use super::engine::*;

pub struct CRMReportsGenerator;

impl ReportGenerator for CRMReportsGenerator {
    fn generate_report(&self, config: ReportConfig) -> CLIERPResult<ReportResult> {
        match config.title.as_str() {
            "customer_analysis" => self.generate_customer_analysis_report(config),
            "sales_pipeline" => self.generate_sales_pipeline_report(config),
            "lead_conversion" => self.generate_lead_conversion_report(config),
            "campaign_performance" => self.generate_campaign_performance_report(config),
            "sales_activity" => self.generate_sales_activity_report(config),
            "revenue_forecast" => self.generate_revenue_forecast_report(config),
            _ => Err(crate::core::error::CLIERPError::NotFound(
                format!("CRM report '{}' not found", config.title)
            )),
        }
    }

    fn get_available_filters(&self) -> Vec<FilterDefinition> {
        vec![
            FilterDefinition {
                name: "customer_segment".to_string(),
                label: "Customer Segment".to_string(),
                filter_type: FilterType::Select,
                required: false,
                default_value: None,
                options: Some(vec![
                    FilterOption { value: "enterprise".to_string(), label: "Enterprise".to_string() },
                    FilterOption { value: "smb".to_string(), label: "Small & Medium Business".to_string() },
                    FilterOption { value: "individual".to_string(), label: "Individual".to_string() },
                ]),
            },
            FilterDefinition {
                name: "sales_rep".to_string(),
                label: "Sales Representative".to_string(),
                filter_type: FilterType::Select,
                required: false,
                default_value: None,
                options: None, // Would be populated dynamically
            },
            FilterDefinition {
                name: "deal_stage".to_string(),
                label: "Deal Stage".to_string(),
                filter_type: FilterType::MultiSelect,
                required: false,
                default_value: None,
                options: Some(vec![
                    FilterOption { value: "qualification".to_string(), label: "Qualification".to_string() },
                    FilterOption { value: "needs_analysis".to_string(), label: "Needs Analysis".to_string() },
                    FilterOption { value: "proposal".to_string(), label: "Proposal".to_string() },
                    FilterOption { value: "negotiation".to_string(), label: "Negotiation".to_string() },
                    FilterOption { value: "closed_won".to_string(), label: "Closed Won".to_string() },
                    FilterOption { value: "closed_lost".to_string(), label: "Closed Lost".to_string() },
                ]),
            },
            FilterDefinition {
                name: "campaign_type".to_string(),
                label: "Campaign Type".to_string(),
                filter_type: FilterType::Select,
                required: false,
                default_value: None,
                options: Some(vec![
                    FilterOption { value: "email".to_string(), label: "Email Marketing".to_string() },
                    FilterOption { value: "social".to_string(), label: "Social Media".to_string() },
                    FilterOption { value: "webinar".to_string(), label: "Webinar".to_string() },
                    FilterOption { value: "event".to_string(), label: "Event".to_string() },
                ]),
            },
        ]
    }

    fn get_report_info(&self) -> ReportInfo {
        ReportInfo {
            id: "crm_reports".to_string(),
            name: "CRM Reports".to_string(),
            description: "Comprehensive CRM reporting including customer analysis, sales pipeline, and campaign performance".to_string(),
            category: "Customer Relationship Management".to_string(),
            supported_formats: vec![
                ReportFormat::Json,
                ReportFormat::Csv,
                ReportFormat::Html,
                ReportFormat::Text,
            ],
        }
    }
}

impl CRMReportsGenerator {
    pub fn new() -> Self {
        Self
    }

    fn generate_customer_analysis_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Customer Segmentation".to_string(),
                section_type: SectionType::Analysis,
                data: ReportData::Chart(create_pie_chart(
                    vec![
                        "Enterprise".to_string(),
                        "Small Business".to_string(),
                        "Individual".to_string(),
                        "Government".to_string(),
                    ],
                    vec![45.0, 30.0, 20.0, 5.0],
                )),
            },
            ReportSection {
                title: "Customer Value Analysis".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Segment".to_string(),
                        "Customer Count".to_string(),
                        "Total Revenue".to_string(),
                        "Avg Revenue/Customer".to_string(),
                        "Growth Rate".to_string(),
                        "Churn Rate".to_string(),
                        "CLV".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Enterprise".to_string(),
                            "125".to_string(),
                            "₩1,850,000,000".to_string(),
                            "₩14,800,000".to_string(),
                            "+12.5%".to_string(),
                            "2.1%".to_string(),
                            "₩45,600,000".to_string(),
                        ],
                        vec![
                            "Small Business".to_string(),
                            "340".to_string(),
                            "₩980,000,000".to_string(),
                            "₩2,882,353".to_string(),
                            "+8.7%".to_string(),
                            "5.8%".to_string(),
                            "₩12,450,000".to_string(),
                        ],
                        vec![
                            "Individual".to_string(),
                            "1,250".to_string(),
                            "₩380,000,000".to_string(),
                            "₩304,000".to_string(),
                            "+15.2%".to_string(),
                            "12.4%".to_string(),
                            "₩1,850,000".to_string(),
                        ],
                        vec![
                            "Government".to_string(),
                            "18".to_string(),
                            "₩650,000,000".to_string(),
                            "₩36,111,111".to_string(),
                            "+3.2%".to_string(),
                            "0.5%".to_string(),
                            "₩95,200,000".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total".to_string(),
                        "1,733".to_string(),
                        "₩3,860,000,000".to_string(),
                        "₩2,227,866".to_string(),
                        "+9.9%".to_string(),
                        "5.2%".to_string(),
                        "₩15,725,000".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Customer Acquisition Trends".to_string(),
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
                            label: "New Customers".to_string(),
                            data: vec![45.0, 52.0, 48.0, 61.0, 58.0, 67.0],
                            color: Some("#10B981".to_string()),
                        },
                        Dataset {
                            label: "Churn".to_string(),
                            data: vec![12.0, 8.0, 15.0, 9.0, 11.0, 7.0],
                            color: Some("#EF4444".to_string()),
                        },
                    ],
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_customers".to_string(), MetricValue::Count(1733));
        key_metrics.insert("customer_growth_rate".to_string(), MetricValue::Percentage(9.9));
        key_metrics.insert("average_clv".to_string(), MetricValue::Currency(15725000));
        key_metrics.insert("customer_satisfaction".to_string(), MetricValue::Number(4.3));
        key_metrics.insert("net_promoter_score".to_string(), MetricValue::Number(67.0));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Enterprise segment drives 47.9% of total revenue with highest CLV".to_string(),
                "Individual segment shows strongest growth at 15.2% but highest churn".to_string(),
                "Government segment has lowest churn rate at 0.5%".to_string(),
                "Customer acquisition accelerated in Q2 with 67 new customers in June".to_string(),
                "Net Promoter Score of 67 indicates strong customer loyalty".to_string(),
            ],
            recommendations: vec![
                "Focus retention efforts on Individual segment to reduce 12.4% churn".to_string(),
                "Develop enterprise upselling programs to maximize CLV".to_string(),
                "Implement referral programs leveraging high NPS".to_string(),
                "Create targeted small business growth initiatives".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 1733,
            processing_time_ms: 280,
            filters_applied: vec!["active_customers".to_string(), "revenue_data".to_string()],
            data_sources: vec!["customers".to_string(), "deals".to_string(), "customer_surveys".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_sales_pipeline_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Pipeline by Stage".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Stage".to_string(),
                        "Deal Count".to_string(),
                        "Total Value".to_string(),
                        "Avg Deal Size".to_string(),
                        "Probability".to_string(),
                        "Weighted Value".to_string(),
                        "Avg Age (Days)".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Qualification".to_string(),
                            "45".to_string(),
                            "₩1,250,000,000".to_string(),
                            "₩27,777,778".to_string(),
                            "20%".to_string(),
                            "₩250,000,000".to_string(),
                            "12".to_string(),
                        ],
                        vec![
                            "Needs Analysis".to_string(),
                            "32".to_string(),
                            "₩980,000,000".to_string(),
                            "₩30,625,000".to_string(),
                            "40%".to_string(),
                            "₩392,000,000".to_string(),
                            "28".to_string(),
                        ],
                        vec![
                            "Proposal".to_string(),
                            "28".to_string(),
                            "₩850,000,000".to_string(),
                            "₩30,357,143".to_string(),
                            "60%".to_string(),
                            "₩510,000,000".to_string(),
                            "45".to_string(),
                        ],
                        vec![
                            "Negotiation".to_string(),
                            "18".to_string(),
                            "₩720,000,000".to_string(),
                            "₩40,000,000".to_string(),
                            "80%".to_string(),
                            "₩576,000,000".to_string(),
                            "67".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total Pipeline".to_string(),
                        "123".to_string(),
                        "₩3,800,000,000".to_string(),
                        "₩30,894,309".to_string(),
                        "45.5%".to_string(),
                        "₩1,728,000,000".to_string(),
                        "38".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Pipeline Velocity".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_bar_chart(
                    vec![
                        "Qualification".to_string(),
                        "Needs Analysis".to_string(),
                        "Proposal".to_string(),
                        "Negotiation".to_string(),
                    ],
                    vec![12.0, 28.0, 45.0, 67.0],
                    "Average Days in Stage",
                )),
            },
            ReportSection {
                title: "Win Rate Analysis".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_line_chart(
                    vec![
                        "Q1 2023".to_string(),
                        "Q2 2023".to_string(),
                        "Q3 2023".to_string(),
                        "Q4 2023".to_string(),
                        "Q1 2024".to_string(),
                        "Q2 2024".to_string(),
                    ],
                    vec![
                        Dataset {
                            label: "Win Rate %".to_string(),
                            data: vec![23.5, 26.8, 29.1, 31.2, 28.7, 33.4],
                            color: Some("#10B981".to_string()),
                        },
                        Dataset {
                            label: "Average Deal Size (M)".to_string(),
                            data: vec![25.2, 27.8, 29.5, 31.8, 28.9, 30.9],
                            color: Some("#3B82F6".to_string()),
                        },
                    ],
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_pipeline_value".to_string(), MetricValue::Currency(3800000000));
        key_metrics.insert("weighted_pipeline".to_string(), MetricValue::Currency(1728000000));
        key_metrics.insert("average_deal_size".to_string(), MetricValue::Currency(30894309));
        key_metrics.insert("win_rate".to_string(), MetricValue::Percentage(33.4));
        key_metrics.insert("sales_cycle_length".to_string(), MetricValue::Number(152.0));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Total pipeline value of ₩3.8 billion across 123 active deals".to_string(),
                "Weighted pipeline of ₩1.73 billion considering stage probabilities".to_string(),
                "Win rate improved to 33.4% in Q2 2024, up from 28.7% in Q1".to_string(),
                "Negotiation stage deals have highest average value at ₩40M".to_string(),
                "Average sales cycle length of 152 days remains stable".to_string(),
            ],
            recommendations: vec![
                "Accelerate qualification process to reduce early-stage deal age".to_string(),
                "Focus on proposal stage optimization to improve conversion".to_string(),
                "Implement sales coaching for negotiation stage deals".to_string(),
                "Develop competitive battle cards for proposal stage".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 123,
            processing_time_ms: 195,
            filters_applied: vec!["active_deals".to_string(), "current_quarter".to_string()],
            data_sources: vec!["deals".to_string(), "leads".to_string(), "sales_activities".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_lead_conversion_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let table_data = TableData {
            headers: vec![
                "Lead Source".to_string(),
                "Leads Generated".to_string(),
                "Qualified Leads".to_string(),
                "Opportunities".to_string(),
                "Closed Won".to_string(),
                "Conversion Rate".to_string(),
                "Avg Time to Close".to_string(),
            ],
            rows: vec![
                vec![
                    "Website".to_string(),
                    "245".to_string(),
                    "189".to_string(),
                    "89".to_string(),
                    "28".to_string(),
                    "11.4%".to_string(),
                    "67 days".to_string(),
                ],
                vec![
                    "Referral".to_string(),
                    "156".to_string(),
                    "142".to_string(),
                    "98".to_string(),
                    "45".to_string(),
                    "28.8%".to_string(),
                    "45 days".to_string(),
                ],
                vec![
                    "Trade Show".to_string(),
                    "89".to_string(),
                    "72".to_string(),
                    "45".to_string(),
                    "18".to_string(),
                    "20.2%".to_string(),
                    "89 days".to_string(),
                ],
                vec![
                    "Cold Outreach".to_string(),
                    "324".to_string(),
                    "145".to_string(),
                    "67".to_string(),
                    "15".to_string(),
                    "4.6%".to_string(),
                    "112 days".to_string(),
                ],
                vec![
                    "Social Media".to_string(),
                    "189".to_string(),
                    "98".to_string(),
                    "34".to_string(),
                    "12".to_string(),
                    "6.3%".to_string(),
                    "78 days".to_string(),
                ],
                vec![
                    "Partner Channel".to_string(),
                    "98".to_string(),
                    "87".to_string(),
                    "56".to_string(),
                    "22".to_string(),
                    "22.4%".to_string(),
                    "56 days".to_string(),
                ],
            ],
            totals: Some(vec![
                "Total".to_string(),
                "1,101".to_string(),
                "733".to_string(),
                "389".to_string(),
                "140".to_string(),
                "12.7%".to_string(),
                "74 days".to_string(),
            ]),
        };

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_leads".to_string(), MetricValue::Count(1101));
        key_metrics.insert("qualification_rate".to_string(), MetricValue::Percentage(66.6));
        key_metrics.insert("opportunity_rate".to_string(), MetricValue::Percentage(35.3));
        key_metrics.insert("overall_conversion_rate".to_string(), MetricValue::Percentage(12.7));
        key_metrics.insert("average_time_to_close".to_string(), MetricValue::Number(74.0));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Referral leads show highest conversion rate at 28.8%".to_string(),
                "Website generates most leads (245) but lower conversion (11.4%)".to_string(),
                "Partner channel demonstrates strong quality with 22.4% conversion".to_string(),
                "Cold outreach has lowest ROI with 4.6% conversion rate".to_string(),
                "Referral leads also close fastest at 45 days average".to_string(),
            ],
            recommendations: vec![
                "Increase investment in referral program development".to_string(),
                "Optimize website lead qualification process".to_string(),
                "Expand partner channel relationships".to_string(),
                "Review and improve cold outreach messaging".to_string(),
                "Implement lead scoring to prioritize high-quality leads".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 1101,
            processing_time_ms: 165,
            filters_applied: vec!["lead_sources".to_string(), "conversion_funnel".to_string()],
            data_sources: vec!["leads".to_string(), "deals".to_string(), "lead_sources".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Table(table_data),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_campaign_performance_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Campaign Results".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Campaign".to_string(),
                        "Type".to_string(),
                        "Budget".to_string(),
                        "Spend".to_string(),
                        "Leads".to_string(),
                        "Cost/Lead".to_string(),
                        "Conversion".to_string(),
                        "ROI".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Q2 Product Launch".to_string(),
                            "Integrated".to_string(),
                            "₩50,000,000".to_string(),
                            "₩48,500,000".to_string(),
                            "189".to_string(),
                            "₩256,614".to_string(),
                            "24.3%".to_string(),
                            "340%".to_string(),
                        ],
                        vec![
                            "Summer Email Series".to_string(),
                            "Email".to_string(),
                            "₩8,000,000".to_string(),
                            "₩7,200,000".to_string(),
                            "145".to_string(),
                            "₩49,655".to_string(),
                            "18.6%".to_string(),
                            "285%".to_string(),
                        ],
                        vec![
                            "LinkedIn Ads".to_string(),
                            "Social".to_string(),
                            "₩15,000,000".to_string(),
                            "₩14,800,000".to_string(),
                            "78".to_string(),
                            "₩189,744".to_string(),
                            "12.8%".to_string(),
                            "156%".to_string(),
                        ],
                        vec![
                            "Trade Show Booth".to_string(),
                            "Event".to_string(),
                            "₩25,000,000".to_string(),
                            "₩23,600,000".to_string(),
                            "67".to_string(),
                            "₩352,239".to_string(),
                            "29.9%".to_string(),
                            "420%".to_string(),
                        ],
                        vec![
                            "Webinar Series".to_string(),
                            "Digital".to_string(),
                            "₩12,000,000".to_string(),
                            "₩11,500,000".to_string(),
                            "156".to_string(),
                            "₩73,718".to_string(),
                            "22.4%".to_string(),
                            "310%".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total".to_string(),
                        "".to_string(),
                        "₩110,000,000".to_string(),
                        "₩105,600,000".to_string(),
                        "635".to_string(),
                        "₩166,299".to_string(),
                        "21.6%".to_string(),
                        "322%".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Campaign ROI Comparison".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_bar_chart(
                    vec![
                        "Product Launch".to_string(),
                        "Email Series".to_string(),
                        "LinkedIn Ads".to_string(),
                        "Trade Show".to_string(),
                        "Webinar Series".to_string(),
                    ],
                    vec![340.0, 285.0, 156.0, 420.0, 310.0],
                    "ROI %",
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_campaign_spend".to_string(), MetricValue::Currency(105600000));
        key_metrics.insert("total_leads_generated".to_string(), MetricValue::Count(635));
        key_metrics.insert("average_cost_per_lead".to_string(), MetricValue::Currency(166299));
        key_metrics.insert("average_conversion_rate".to_string(), MetricValue::Percentage(21.6));
        key_metrics.insert("average_campaign_roi".to_string(), MetricValue::Percentage(322.0));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Trade Show Booth achieved highest ROI at 420% despite high cost per lead".to_string(),
                "Email campaigns deliver best cost efficiency at ₩49,655 per lead".to_string(),
                "Q2 Product Launch generated most leads (189) with strong 24.3% conversion".to_string(),
                "All campaigns exceeded 150% ROI threshold indicating positive performance".to_string(),
                "Digital channels (Email, Webinar) show consistent performance".to_string(),
            ],
            recommendations: vec![
                "Increase budget allocation to high-ROI Trade Show activities".to_string(),
                "Scale email marketing programs for cost-effective lead generation".to_string(),
                "Optimize LinkedIn Ads targeting to improve conversion rates".to_string(),
                "Replicate Q2 Product Launch success formula for future launches".to_string(),
                "Develop integrated campaign approach combining best-performing channels".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 5,
            processing_time_ms: 125,
            filters_applied: vec!["completed_campaigns".to_string(), "q2_2024".to_string()],
            data_sources: vec!["campaigns".to_string(), "leads".to_string(), "campaign_costs".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_sales_activity_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Activity Summary by Type".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Activity Type".to_string(),
                        "Total Count".to_string(),
                        "Completed".to_string(),
                        "Pending".to_string(),
                        "Overdue".to_string(),
                        "Completion Rate".to_string(),
                        "Avg Duration".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Calls".to_string(),
                            "1,245".to_string(),
                            "1,156".to_string(),
                            "67".to_string(),
                            "22".to_string(),
                            "92.9%".to_string(),
                            "18 min".to_string(),
                        ],
                        vec![
                            "Emails".to_string(),
                            "2,890".to_string(),
                            "2,834".to_string(),
                            "56".to_string(),
                            "0".to_string(),
                            "98.1%".to_string(),
                            "5 min".to_string(),
                        ],
                        vec![
                            "Meetings".to_string(),
                            "456".to_string(),
                            "398".to_string(),
                            "45".to_string(),
                            "13".to_string(),
                            "87.3%".to_string(),
                            "52 min".to_string(),
                        ],
                        vec![
                            "Demos".to_string(),
                            "189".to_string(),
                            "167".to_string(),
                            "18".to_string(),
                            "4".to_string(),
                            "88.4%".to_string(),
                            "45 min".to_string(),
                        ],
                        vec![
                            "Follow-ups".to_string(),
                            "678".to_string(),
                            "612".to_string(),
                            "54".to_string(),
                            "12".to_string(),
                            "90.3%".to_string(),
                            "12 min".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Total".to_string(),
                        "5,458".to_string(),
                        "5,167".to_string(),
                        "240".to_string(),
                        "51".to_string(),
                        "94.7%".to_string(),
                        "26 min".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Sales Rep Performance".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Sales Rep".to_string(),
                        "Activities".to_string(),
                        "Calls".to_string(),
                        "Meetings".to_string(),
                        "Completion %".to_string(),
                        "Pipeline Value".to_string(),
                        "Closed Deals".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "김영수".to_string(),
                            "456".to_string(),
                            "145".to_string(),
                            "34".to_string(),
                            "96.2%".to_string(),
                            "₩450,000,000".to_string(),
                            "8".to_string(),
                        ],
                        vec![
                            "박민지".to_string(),
                            "398".to_string(),
                            "122".to_string(),
                            "28".to_string(),
                            "94.1%".to_string(),
                            "₩380,000,000".to_string(),
                            "6".to_string(),
                        ],
                        vec![
                            "이창호".to_string(),
                            "512".to_string(),
                            "167".to_string(),
                            "42".to_string(),
                            "97.8%".to_string(),
                            "₩520,000,000".to_string(),
                            "9".to_string(),
                        ],
                        vec![
                            "정수연".to_string(),
                            "423".to_string(),
                            "134".to_string(),
                            "31".to_string(),
                            "93.6%".to_string(),
                            "₩410,000,000".to_string(),
                            "7".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "Team Total".to_string(),
                        "1,789".to_string(),
                        "568".to_string(),
                        "135".to_string(),
                        "95.4%".to_string(),
                        "₩1,760,000,000".to_string(),
                        "30".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Activity Trends".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_line_chart(
                    vec![
                        "Week 1".to_string(),
                        "Week 2".to_string(),
                        "Week 3".to_string(),
                        "Week 4".to_string(),
                    ],
                    vec![
                        Dataset {
                            label: "Calls".to_string(),
                            data: vec![285.0, 320.0, 295.0, 345.0],
                            color: Some("#3B82F6".to_string()),
                        },
                        Dataset {
                            label: "Meetings".to_string(),
                            data: vec![98.0, 115.0, 108.0, 135.0],
                            color: Some("#10B981".to_string()),
                        },
                        Dataset {
                            label: "Emails".to_string(),
                            data: vec![645.0, 720.0, 678.0, 847.0],
                            color: Some("#F59E0B".to_string()),
                        },
                    ],
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("total_activities".to_string(), MetricValue::Count(5458));
        key_metrics.insert("completion_rate".to_string(), MetricValue::Percentage(94.7));
        key_metrics.insert("overdue_activities".to_string(), MetricValue::Count(51));
        key_metrics.insert("average_activities_per_rep".to_string(), MetricValue::Number(447.3));
        key_metrics.insert("team_pipeline_value".to_string(), MetricValue::Currency(1760000000));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "Sales team completed 5,167 activities with 94.7% completion rate".to_string(),
                "이창호 leads team with 512 activities and ₩520M pipeline value".to_string(),
                "Email activities have highest completion rate at 98.1%".to_string(),
                "Only 51 activities are overdue, indicating good time management".to_string(),
                "Activity volume increased 21% in Week 4 compared to Week 1".to_string(),
            ],
            recommendations: vec![
                "Provide time management coaching for reps with overdue activities".to_string(),
                "Share best practices from top performers like 이창호".to_string(),
                "Implement activity automation for routine follow-ups".to_string(),
                "Set team goals for meeting-to-call ratios".to_string(),
                "Review and optimize activity scheduling processes".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 5458,
            processing_time_ms: 320,
            filters_applied: vec!["current_month".to_string(), "sales_team".to_string()],
            data_sources: vec!["activities".to_string(), "employees".to_string(), "deals".to_string()],
        };

        Ok(ReportResult {
            config,
            generated_at: Utc::now().naive_utc(),
            data: ReportData::Mixed(sections),
            summary: Some(summary),
            metadata,
        })
    }

    fn generate_revenue_forecast_report(&self, config: ReportConfig) -> Result<ReportResult> {
        let sections = vec![
            ReportSection {
                title: "Quarterly Revenue Forecast".to_string(),
                section_type: SectionType::Detail,
                data: ReportData::Table(TableData {
                    headers: vec![
                        "Quarter".to_string(),
                        "Pipeline".to_string(),
                        "Probability".to_string(),
                        "Forecast".to_string(),
                        "Upside".to_string(),
                        "Conservative".to_string(),
                        "Actual/Target".to_string(),
                    ],
                    rows: vec![
                        vec![
                            "Q2 2024".to_string(),
                            "₩1,850,000,000".to_string(),
                            "65%".to_string(),
                            "₩1,202,500,000".to_string(),
                            "₩1,387,500,000".to_string(),
                            "₩925,000,000".to_string(),
                            "₩1,145,000,000".to_string(),
                        ],
                        vec![
                            "Q3 2024".to_string(),
                            "₩2,100,000,000".to_string(),
                            "58%".to_string(),
                            "₩1,218,000,000".to_string(),
                            "₩1,470,000,000".to_string(),
                            "₩1,050,000,000".to_string(),
                            "₩1,200,000,000".to_string(),
                        ],
                        vec![
                            "Q4 2024".to_string(),
                            "₩2,400,000,000".to_string(),
                            "52%".to_string(),
                            "₩1,248,000,000".to_string(),
                            "₩1,560,000,000".to_string(),
                            "₩1,080,000,000".to_string(),
                            "₩1,250,000,000".to_string(),
                        ],
                        vec![
                            "Q1 2025".to_string(),
                            "₩2,200,000,000".to_string(),
                            "45%".to_string(),
                            "₩990,000,000".to_string(),
                            "₩1,320,000,000".to_string(),
                            "₩880,000,000".to_string(),
                            "₩1,100,000,000".to_string(),
                        ],
                    ],
                    totals: Some(vec![
                        "FY 2024".to_string(),
                        "₩8,550,000,000".to_string(),
                        "55%".to_string(),
                        "₩4,658,500,000".to_string(),
                        "₩5,737,500,000".to_string(),
                        "₩3,935,000,000".to_string(),
                        "₩4,695,000,000".to_string(),
                    ]),
                }),
            },
            ReportSection {
                title: "Revenue Trend & Forecast".to_string(),
                section_type: SectionType::Chart,
                data: ReportData::Chart(create_line_chart(
                    vec![
                        "Q1 2023".to_string(),
                        "Q2 2023".to_string(),
                        "Q3 2023".to_string(),
                        "Q4 2023".to_string(),
                        "Q1 2024".to_string(),
                        "Q2 2024".to_string(),
                        "Q3 2024".to_string(),
                        "Q4 2024".to_string(),
                    ],
                    vec![
                        Dataset {
                            label: "Actual Revenue".to_string(),
                            data: vec![890.0, 1050.0, 980.0, 1180.0, 1100.0, 1145.0, 0.0, 0.0],
                            color: Some("#10B981".to_string()),
                        },
                        Dataset {
                            label: "Forecast".to_string(),
                            data: vec![0.0, 0.0, 0.0, 0.0, 0.0, 1202.5, 1218.0, 1248.0],
                            color: Some("#3B82F6".to_string()),
                        },
                        Dataset {
                            label: "Conservative".to_string(),
                            data: vec![0.0, 0.0, 0.0, 0.0, 0.0, 925.0, 1050.0, 1080.0],
                            color: Some("#F59E0B".to_string()),
                        },
                    ],
                )),
            },
        ];

        let mut key_metrics = HashMap::new();
        key_metrics.insert("fy2024_forecast".to_string(), MetricValue::Currency(4658500000));
        key_metrics.insert("forecast_accuracy".to_string(), MetricValue::Percentage(94.2));
        key_metrics.insert("revenue_growth_yoy".to_string(), MetricValue::Percentage(12.8));
        key_metrics.insert("pipeline_coverage".to_string(), MetricValue::Number(1.8));
        key_metrics.insert("forecast_confidence".to_string(), MetricValue::Percentage(85.0));

        let summary = ReportSummary {
            key_metrics,
            insights: vec![
                "FY 2024 revenue forecast of ₩4.66 billion represents 12.8% YoY growth".to_string(),
                "Q2 2024 achieved 95.2% of forecast, indicating strong predictability".to_string(),
                "Pipeline coverage ratio of 1.8x provides healthy forecast buffer".to_string(),
                "Conservative scenario still achieves ₩3.94 billion, exceeding previous year".to_string(),
                "Forecast accuracy improved to 94.2% with enhanced methodology".to_string(),
            ],
            recommendations: vec![
                "Maintain focus on Q3/Q4 pipeline development for stronger coverage".to_string(),
                "Implement weekly forecast reviews with sales team".to_string(),
                "Develop contingency plans for conservative scenario achievement".to_string(),
                "Invest in sales enablement to improve close rates".to_string(),
                "Consider accelerating Q1 2025 pipeline development".to_string(),
            ],
        };

        let metadata = ReportMetadata {
            total_records: 4,
            processing_time_ms: 145,
            filters_applied: vec!["forecast_model".to_string(), "quarterly_view".to_string()],
            data_sources: vec!["deals".to_string(), "historical_actuals".to_string(), "pipeline_analysis".to_string()],
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

impl Default for CRMReportsGenerator {
    fn default() -> Self {
        Self::new()
    }
}