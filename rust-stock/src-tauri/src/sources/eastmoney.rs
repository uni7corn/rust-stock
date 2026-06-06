// eastmoney.rs — 东方财富数据源
// canonical → secid 转换在这里完成（前端不再关心各源的私有格式）。

use super::QuoteSource;
use crate::quote::Quote;

pub struct EastmoneySource;

/// canonical → 东财 secid（前缀：1.=沪 0.=深 100.=国际指数）
pub fn to_secid(code: &str) -> Option<String> {
    match code {
        "int_dji" => Some("100.DJIA".into()),
        "int_nasdaq" => Some("100.NDX".into()),
        c if c.starts_with("sh") && c.len() == 8 => Some(format!("1.{}", &c[2..])),
        c if c.starts_with("sz") && c.len() == 8 => Some(format!("0.{}", &c[2..])),
        _ => None,
    }
}

/// 股票搜索命中（东财 suggest 接口，2026-06-06 实测）
#[derive(Debug, serde::Serialize)]
pub struct SearchHit {
    pub code: String,   // 统一格式 sh600519
    pub name: String,   // 贵州茅台
    pub market: String, // 沪A / 深A
}

/// 解析 suggest 返回：{"QuotationCodeTable":{"Data":[{"Code":"600519","Name":"贵州茅台",
///   "Classify":"AStock","MktNum":"1","SecurityTypeName":"沪A","PinYin":"GZMT",…}]}}
/// 只收 A 股个股（Classify==AStock），支持名称/代码/拼音首字母搜索。
pub fn parse_search(body: &str) -> Vec<SearchHit> {
    let v: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    let list = match v["QuotationCodeTable"]["Data"].as_array() {
        Some(l) => l,
        None => return vec![],
    };
    list.iter()
        .filter_map(|d| {
            if d["Classify"].as_str() != Some("AStock") {
                return None; // 过滤指数/基金/港美股
            }
            let code6 = d["Code"].as_str()?;
            let prefix = match d["MktNum"].as_str()? {
                "1" => "sh",
                "0" => "sz",
                _ => return None,
            };
            Some(SearchHit {
                code: format!("{prefix}{code6}"),
                name: d["Name"].as_str().unwrap_or("").to_string(),
                market: d["SecurityTypeName"].as_str().unwrap_or("").to_string(),
            })
        })
        .take(8)
        .collect()
}

#[cfg(feature = "net")]
pub async fn search(keyword: &str) -> Result<Vec<SearchHit>, String> {
    let resp = reqwest::Client::new()
        .get("https://searchapi.eastmoney.com/api/suggest/get")
        .query(&[
            ("input", keyword),
            ("type", "14"),
            ("token", "D43BF722C8E33BDC906FB84D85E326E8"), // 东财网页端公开常量
            ("count", "10"),
        ])
        .send()
        .await
        .map_err(|e| format!("搜索请求失败: {e}"))?;
    let text = resp.text().await.map_err(|e| e.to_string())?;
    Ok(parse_search(&text))
}

#[cfg(not(feature = "net"))]
pub async fn search(_keyword: &str) -> Result<Vec<SearchHit>, String> {
    Err("net feature 未启用".into())
}

#[async_trait::async_trait]
impl QuoteSource for EastmoneySource {
    fn id(&self) -> &'static str {
        "eastmoney"
    }
    fn display_name(&self) -> &'static str {
        "东方财富"
    }

    #[cfg(feature = "net")]
    async fn fetch(&self, codes: &[String]) -> Result<Vec<Quote>, String> {
        let secids: Vec<String> = codes.iter().filter_map(|c| to_secid(c)).collect();
        if secids.is_empty() {
            return Err("没有可识别的代码".into());
        }
        let refs: Vec<&str> = secids.iter().map(|s| s.as_str()).collect();
        crate::quote::fetch_eastmoney(&refs).await
    }

    #[cfg(not(feature = "net"))]
    async fn fetch(&self, _codes: &[String]) -> Result<Vec<Quote>, String> {
        Err("net feature 未启用".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_search() {
        // 按 2026-06-06 实测返回的真实形状
        let raw = r#"{"QuotationCodeTable":{"Data":[
            {"Code":"600519","Name":"贵州茅台","PinYin":"GZMT","Classify":"AStock","SecurityTypeName":"沪A","MktNum":"1","QuoteID":"1.600519"},
            {"Code":"000001","Name":"平安银行","PinYin":"PAYH","Classify":"AStock","SecurityTypeName":"深A","MktNum":"0","QuoteID":"0.000001"},
            {"Code":"00700","Name":"腾讯控股","Classify":"HKStock","SecurityTypeName":"港股","MktNum":"116"}
        ],"Status":0}}"#;
        let hits = parse_search(raw);
        assert_eq!(hits.len(), 2); // 港股被过滤
        assert_eq!(hits[0].code, "sh600519");
        assert_eq!(hits[0].name, "贵州茅台");
        assert_eq!(hits[1].code, "sz000001");
        assert_eq!(hits[1].market, "深A");
        assert_eq!(parse_search("not json").len(), 0);
    }

    #[test]
    fn test_to_secid() {
        assert_eq!(to_secid("sh600519").unwrap(), "1.600519");
        assert_eq!(to_secid("sh000001").unwrap(), "1.000001");
        assert_eq!(to_secid("sz000001").unwrap(), "0.000001");
        assert_eq!(to_secid("int_dji").unwrap(), "100.DJIA");
        assert_eq!(to_secid("int_nasdaq").unwrap(), "100.NDX");
        assert!(to_secid("xx123").is_none());
        assert!(to_secid("sh12345").is_none()); // 长度不对
    }
}
