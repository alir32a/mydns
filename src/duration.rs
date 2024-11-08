use std::io::{Result, Error, ErrorKind};
use std::ops::Add;
use std::time::Duration;

static MINUTE: u64 = 60;
static HOUR: u64 = 60 * MINUTE;
static DAY: u64 = 24 * HOUR;

pub(crate) enum TimeUnit {
    NanoSecond,
    MicroSecond,
    MilliSecond,
    Second,
    Minute,
    Hour,
    Day,
}

impl TimeUnit {
    pub fn to_duration(&self, n: u64) -> Duration {
        match self {
            TimeUnit::NanoSecond => Duration::from_nanos(n),
            TimeUnit::MicroSecond => Duration::from_micros(n),
            TimeUnit::MilliSecond => Duration::from_millis(n),
            TimeUnit::Second => Duration::from_secs(n),
            TimeUnit::Minute => Duration::from_secs(MINUTE * n),
            TimeUnit::Hour => Duration::from_secs(HOUR * n),
            TimeUnit::Day => Duration::from_secs(DAY * n)
        }
    }

    pub fn from(s: &str) -> Result<Self> {
        match s {
            "d" | "D" => Ok(Self::Day),
            "h" | "H" => Ok(Self::Hour),
            "m" | "M" => Ok(Self::Minute),
            "s" | "S" => Ok(Self::Second),
            "ms" => Ok(Self::MilliSecond),
            "µs" | "us" => Ok(Self::MicroSecond),
            "ns" => Ok(Self::NanoSecond),
            _ => Err(Error::new(ErrorKind::InvalidInput, format!("{} is an invalid time unit", s)))
        }
    }
}

struct SplitUnits(Vec<u64>, Vec<String>);

pub fn parse(s: &str) -> Result<Duration> {
    let mut res = Duration::ZERO;

    let split_units = split_by_nums(s)?;

    for (i, num) in split_units.0.iter().enumerate() {
        let unit = TimeUnit::from(&split_units.1[i])?;

        res = res.add(unit.to_duration(*num));
    }

    Ok(res)
}

fn split_by_nums(s: &str) -> Result<SplitUnits> {
    let nums = split_nums(s);
    let units = split_str(s);

    if nums.len() != units.len() {
        return Err(Error::new(ErrorKind::InvalidInput, format!("{} is an invalid duration", s)));
    }

    Ok(SplitUnits(nums, units))
}

fn split_nums(s: &str) -> Vec<u64> {
    let mut res = Vec::new();
    
    let mut temp: u64 = 0;
    for ch in s.chars() {
        if ch.is_ascii_digit() {
            if temp == 0 {
                if let Some(d) = ch.to_digit(10) {
                    temp = d as u64;
                }
            } else {
                if let Some(d) = ch.to_digit(10) {
                    temp *= 10;
                    temp += d as u64;
                }
            }
        } else {
            if temp != 0 {
                res.push(temp)
            }
            
            temp = 0;
        }
    }
    
    if temp != 0 {
        res.push(temp);
    }
    
    res
}

fn split_str(s: &str) -> Vec<String> {
    let mut res = Vec::new();
    
    let mut temp = String::new();
    for ch in s.chars() {
        if ch.is_ascii_alphabetic() {
            temp.push(ch)
        } else {
            if !temp.is_empty() {
                res.push(temp.clone());
                
                temp.clear()
            }
        }
    }
    
    if !temp.is_empty() {
        res.push(temp)
    }
    
    res
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn time_units() {
        assert_eq!(parse("12s").ok(), Some(Duration::from_secs(12)));
        assert_eq!(parse("1m10s").ok(), Some(Duration::from_secs(70)));
        assert_eq!(parse("1h15m10s").ok(), Some(Duration::from_secs(HOUR + (15 * MINUTE) + 10)));
        
        assert_eq!(parse("1G").ok(), None);
        assert_eq!(parse("1h34m23g").ok(), None);
    }
}